use super::BufferKey;

/// Used to inform the implementation of how it should be bound.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum BufferType {
    Vertex,
    Index,
}

impl Into<u32> for BufferType {
    fn into(self) -> u32 {
        match self {
            BufferType::Vertex => glow::ARRAY_BUFFER,
            BufferType::Index => glow::ELEMENT_ARRAY_BUFFER,
        }
    }
}

/// Used to hint to the implementation how frequently the user will be changing the buffer's data.
/// * `Static`: The user will set the data once.
/// * `Dynamic`: The user will set the data occasionally.
/// * `Stream`: The user will be changing the data after every use. Or almost every use.
#[derive(Copy, Clone, Debug)]
pub enum Usage {
    Stream,
    Static,
    Dynamic,
}

impl Usage {
    pub fn to_gl(self) -> u32 {
        match self {
            Usage::Stream => glow::STREAM_DRAW,
            Usage::Static => glow::STATIC_DRAW,
            Usage::Dynamic => glow::DYNAMIC_DRAW,
        }
    }
}

/// A memory map between a CPU and GPU buffer.
///
/// This implementation, while safe, only operates on bytes to better mirror GPU buffers. It is best
/// used through a [`Mesh`](graphics::mesh::Mesh) to provide information on how the data is laid out
/// internally and allow the use of more types and structures.
///
/// This buffer is not resizable. All operations are sized in bytes.
#[derive(Clone, Debug)]
pub struct Buffer {
    size: usize,
    handle: BufferKey,
    buffer_type: BufferType,
    usage: Usage,
}

impl Buffer {
    /// Constructs an empty buffer of `size` bytes.
    pub fn new(
        ctx: &mut super::Context,
        size: usize,
        buffer_type: BufferType,
        usage: Usage,
    ) -> Result<Self, super::GraphicsError> {
        let handle = ctx.new_buffer(size, buffer_type, usage, None)?;
        Ok(Self {
            size,
            handle,
            buffer_type,
            usage,
        })
    }

    /// Constructs a buffer of the size and contents of the passed in the Vec.
    pub fn with_data(
        ctx: &mut super::Context,
        data: &[u8],
        buffer_type: BufferType,
        usage: Usage,
    ) -> Result<Self, super::GraphicsError> {
        let size = data.len();
        let handle = ctx.new_buffer(size, buffer_type, usage, Some(data))?;
        Ok(Self {
            size,
            handle,
            buffer_type,
            usage,
        })
    }

    /// Returns an identifier that can be used with the graphics context to retrieve the raw GPU
    /// buffer handle.
    pub fn handle(&self) -> BufferKey {
        self.handle
    }

    /// The buffer's capacity/size. Since it's not resizable these concepts are the same.
    pub fn size(&self) -> usize {
        self.size
    }

    /// The buffer's type.
    pub fn buffer_type(&self) -> BufferType {
        self.buffer_type
    }

    /// The buffer's usage.
    pub fn usage(&self) -> Usage {
        self.usage
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ModifiedRange<D> {
    pub offset: D,
    pub size: D,
}

pub struct Mapped<T, D> {
    inner: T,
    memory_map: ndarray::Array<u8, D>,
    modified_range: Option<ModifiedRange<D>>,
}

impl<T, D> Mapped<T, D>
where
    D: ndarray::Dimension,
{
    pub fn with_shape<S>(inner: T, shape: S) -> Self
    where
        S: ndarray::ShapeBuilder<Dim = D>,
    {
        Self {
            inner,
            memory_map: ndarray::Array::default(shape),
            modified_range: None,
        }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn memory_map(&self) -> &[u8] {
        self.memory_map.as_slice_memory_order().unwrap()
    }
}

impl<T> Mapped<T, ndarray::Ix1> {
    pub fn from_vec(inner: T, vec: Vec<u8>) -> Self {
        Self {
            inner,
            memory_map: vec.into(),
            modified_range: None,
        }
    }

    /// Write new data into the buffer and adjust it's dirty range accordingly.
    ///
    /// This function will panic if the buffer overflows.
    pub fn write(&mut self, data: &[u8], offset: usize) {
        self.memory_map.as_slice_memory_order_mut().unwrap()[offset..(offset + data.len())]
            .copy_from_slice(data);
        self.set_modified_range(offset, data.len());
    }

    pub fn modified_range(&self) -> Option<ModifiedRange<usize>> {
        self.modified_range.map(|range| ModifiedRange {
            offset: range.offset[0],
            size: range.size[0],
        })
    }

    /// Sets the dirty range of this buffer. This marks how much of the data is synced to the GPU
    /// when this buffer is unmapped.
    fn set_modified_range(&mut self, offset: usize, modified_size: usize) {
        let range = self.modified_range.get_or_insert(ModifiedRange {
            offset: ndarray::Ix1(0),
            size: ndarray::Ix1(0),
        });
        // We're being conservative right now by internally marking the whole range
        // from the start of section a to the end of section b as modified if both
        // a and b are marked as modified.
        let old_range_end = range.offset + range.size;
        range.offset = ndarray::Ix1(std::cmp::min(range.offset[0], offset));

        let new_range_end = std::cmp::max(offset + modified_size, old_range_end[0]);
        range.size = ndarray::Ix1(new_range_end - range.offset[0]);
    }
}

pub type MappedBuffer = Mapped<Buffer, ndarray::Ix1>;
impl MappedBuffer {
    pub fn with_buffer(
        ctx: &mut super::Context,
        size: usize,
        buffer_type: BufferType,
        usage: Usage,
    ) -> Result<Self, super::GraphicsError> {
        let inner = Buffer::new(ctx, size, buffer_type, usage)?;
        let memory_map = ndarray::Array1::from(vec![0u8; inner.size()]);
        Ok(Self {
            inner,
            memory_map,
            modified_range: None,
        })
    }

    pub fn unmap(&mut self, ctx: &mut super::Context) {
        ctx.unmap_buffer(self);
        self.modified_range = None;
    }
}

// pub trait MappedBufferTrait<I> {
//     type UnmappedBuffer;
//
//     fn set(&mut self, index: I, v: u8);
//     fn get(&self, index: I) -> Option<&u8>;
//     fn unmap(&mut self, ctx: &mut super::Context) -> &Self::UnmappedBuffer;
// }
//
// struct MappedBuffer<D> {
//     inner: Buffer,
//     memory_map: ndarray::Array<u8, D>
// }
//
// impl<I> MappedBufferTrait<I> for MappedBuffer<ndarray::Ix1> where I: ndarray::NdIndex<ndarray::Ix1> {
//     type UnmappedBuffer = Buffer;
//
//     fn set(&mut self, index: I, v: u8) {
//         self.memory_map[index] = v;
//     }
//
//     fn get(&self, index: I) -> Option<&u8> {
//         self.memory_map.get(index)
//     }
//
//     fn unmap(&mut self, ctx: &mut super::Context) -> &Self::UnmappedBuffer {
//         ctx.unmap_buffer(&mut self.inner);
//         &self.inner
//     }
// }

// impl<I, D> MappedBufferTrait<I> for MappedBuffer<D> where D: ndarray::Dimension, I: ndarray::NdIndex<D> {
//     fn set(&mut self, index: I, v: u8) {
//         self.memory_map[index] = v;
//     }
//
//     fn get(&self, index: I) -> Option<&u8> {
//         self.memory_map.get(index)
//     }
// }
