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
pub struct Buffer {
    memory_map: Box<[u8]>,
    modified_offset: usize,
    modified_size: usize,
    handle: BufferKey,
    buffer_type: BufferType,
    usage: Usage,
}

impl Buffer {
    /// Constructs an empty buffer of `size` bytes.
    pub fn new(
        gl: &mut super::Context,
        size: usize,
        buffer_type: BufferType,
        usage: Usage,
    ) -> Result<Self, super::GraphicsError> {
        Self::from_vec(gl, vec![0u8; size], buffer_type, usage)
    }

    /// Constructs a buffer of the size and contents of the passed in the Vec.
    pub fn from_vec(
        gl: &mut super::Context,
        vec: Vec<u8>,
        buffer_type: BufferType,
        usage: Usage,
    ) -> Result<Self, super::GraphicsError> {
        let size = vec.len();
        let handle = gl.new_buffer(size, buffer_type, usage)?;
        let memory_map = vec.into_boxed_slice();
        Ok(Self {
            memory_map,
            modified_offset: 0,
            modified_size: size,
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

    /// Sets the dirty range of this buffer. This marks how much of the data is synced to the GPU
    /// when this buffer is unmapped.
    pub fn set_modified_range(&mut self, offset: usize, modified_size: usize) {
        // We're being conservative right now by internally marking the whole range
        // from the start of section a to the end of section b as modified if both
        // a and b are marked as modified.
        let old_range_end = self.modified_offset + self.modified_size;
        self.modified_offset = std::cmp::min(self.modified_offset, offset);

        let new_range_end = std::cmp::max(offset + modified_size, old_range_end);
        self.modified_size = new_range_end - self.modified_offset;
    }

    /// Clears the dirty range.
    pub fn reset_modified_range(&mut self) {
        self.modified_offset = 0;
        self.modified_size = 0;
    }

    /// The buffer's capacity/size. Since it's not resizable these concepts are the same.
    pub fn size(&self) -> usize {
        self.memory_map.len()
    }

    /// The size of the dirty range.
    pub fn modified_size(&self) -> usize {
        self.modified_size
    }

    /// The offset of the start of the dirty range.
    pub fn modified_offset(&self) -> usize {
        self.modified_offset
    }

    /// The underlying memory map.
    pub fn memory_map(&self) -> &[u8] {
        &self.memory_map
    }

    /// The buffer's type.
    pub fn buffer_type(&self) -> BufferType {
        self.buffer_type
    }

    /// The buffer's usage.
    pub fn usage(&self) -> Usage {
        self.usage
    }

    /// Write new data into the buffer and adjust it's dirty range accordingly.
    ///
    /// This function will panic if the buffer overflows.
    pub fn write(&mut self, data: &[u8], offset: usize) {
        assert!(
            data.len() + offset <= self.size(),
            "Overfilled buffer memory map. Length ({}) + offset ({}) > {}",
            data.len(),
            offset,
            self.size()
        );
        self.memory_map[offset..(offset + data.len())].copy_from_slice(data);
        self.set_modified_range(offset, data.len());
    }
}
