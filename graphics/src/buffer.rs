use super::BufferKey;

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

pub struct Buffer {
    memory_map: Box<[u8]>,
    modified_offset: usize,
    modified_size: usize,
    handle: BufferKey,
    buffer_type: BufferType,
    usage: Usage,
}

impl Buffer {
    pub fn new(
        gl: &mut super::Context,
        size: usize,
        buffer_type: BufferType,
        usage: Usage,
    ) -> Result<Self, super::GraphicsError> {
        let handle = gl.new_buffer(size, buffer_type, usage)?;
        let memory_map = vec![0u8; size].into_boxed_slice();
        Ok(Self {
            memory_map,
            modified_offset: 0,
            modified_size: 0,
            handle,
            buffer_type,
            usage,
        })
    }

    pub fn handle(&self) -> BufferKey {
        self.handle
    }

    pub fn set_modified_range(&mut self, offset: usize, modified_size: usize) {
        // We're being conservative right now by internally marking the whole range
        // from the start of section a to the end of section b as modified if both
        // a and b are marked as modified.
        let old_range_end = self.modified_offset + self.modified_size;
        self.modified_offset = std::cmp::min(self.modified_offset, offset);

        let new_range_end = std::cmp::max(offset + modified_size, old_range_end);
        self.modified_size = new_range_end - self.modified_offset;
    }

    pub fn reset_modified_range(&mut self) {
        self.modified_offset = 0;
        self.modified_size = 0;
    }

    pub fn size(&self) -> usize {
        self.memory_map.len()
    }

    pub fn modified_size(&self) -> usize {
        self.modified_size
    }

    pub fn modified_offset(&self) -> usize {
        self.modified_offset
    }

    pub fn memory_map(&self) -> &[u8] {
        &self.memory_map
    }

    pub fn buffer_type(&self) -> BufferType {
        self.buffer_type
    }

    pub fn usage(&self) -> Usage {
        self.usage
    }

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
