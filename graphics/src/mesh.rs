use super::{
    buffer::{BufferType, Usage},
    vertex::Vertex,
    BufferKey, Context,
};
use std::cell::RefCell;
use std::rc::Rc;

pub type Index = u32;
const INDEX_GL: u32 = glow::UNSIGNED_INT;

pub enum IndexType {
    UnsignedByte(u8),
    UnsignedShort(u16),
    UnsignedInt(u32),
}

impl IndexType {
    pub fn to_gl(&self) -> u32 {
        match self {
            IndexType::UnsignedByte(_) => glow::UNSIGNED_BYTE,
            IndexType::UnsignedShort(_) => glow::UNSIGNED_SHORT,
            IndexType::UnsignedInt(_) => glow::UNSIGNED_INT,
        }
    }
}

pub struct Mesh<T> {
    gl: Rc<RefCell<Context>>,
    vbo: BufferKey,
    ibo: BufferKey,
    use_indices: bool,
    draw_range: Option<std::ops::Range<usize>>,
    vertex_marker: std::marker::PhantomData<T>,
    draw_mode: super::DrawMode,
}

impl<T> Mesh<T>
where
    T: Vertex,
{
    pub fn new(gl: Rc<RefCell<Context>>, size: usize) -> Self {
        let vbo = gl.borrow_mut().new_buffer(
            size * std::mem::size_of::<T>(),
            BufferType::Vertex,
            Usage::Dynamic,
        );
        let ibo = gl.borrow_mut().new_buffer(
            size * std::mem::size_of::<Index>(),
            BufferType::Index,
            Usage::Dynamic,
        );
        Self {
            gl,
            vbo,
            ibo,
            use_indices: false,
            draw_range: None,
            draw_mode: super::DrawMode::Triangles,
            vertex_marker: std::marker::PhantomData,
        }
    }

    fn set_buffer<V>(&mut self, buffer: BufferKey, data: &[V], offset: usize)
    where
        V: Sized,
    {
        let mut gl = self.gl.borrow_mut();
        let buffer = gl.get_buffer_mut(buffer).unwrap();
        buffer.write(
            unsafe {
                std::slice::from_raw_parts(
                    data.as_ptr() as *const u8,
                    data.len() * std::mem::size_of::<V>(),
                )
            },
            offset * std::mem::size_of::<V>(),
        );
    }

    pub fn set_vertices(&mut self, vertices: &[T], offset: usize) {
        self.set_buffer(self.vbo, vertices, offset);
    }

    pub fn set_indices(&mut self, indices: &[Index], offset: usize) {
        self.use_indices = true;
        self.set_buffer(self.ibo, indices, offset);
    }

    pub fn set_draw_range(&mut self, draw_range: Option<std::ops::Range<usize>>) {
        self.draw_range = draw_range;
    }

    pub fn draw(&mut self) {
        self.draw_instanced(1);
    }

    pub fn draw_instanced(&mut self, instance_count: usize) {
        if let Some(draw_range) = &self.draw_range {
            if !(draw_range.start <= draw_range.end) {
                return;
            }
        }

        let mut gl = self.gl.borrow_mut();
        gl.bind_buffer(self.vbo);

        let stride = std::mem::size_of::<T>();
        let bindings = T::build_bindings();

        let shader = gl
            .get_shader(gl.get_active_shader().expect("No active shader."))
            .unwrap();
        let mut desired_attribute_state = 0u32;
        let attributes = shader
            .attributes()
            .iter()
            .map(|attr| {
                let vertex_format = bindings
                    .iter()
                    .find(|binding| binding.name == attr.name.as_str());
                if let Some(_vertex_format) = vertex_format {
                    desired_attribute_state |= 1 << attr.location;
                }
                vertex_format.map(|v| (v, self.vbo))
            })
            .filter(Option::is_some)
            .map(Option::unwrap)
            .collect::<Vec<_>>();
        gl.set_vertex_attributes(desired_attribute_state, stride, &attributes);

        gl.unmap_buffer(self.vbo);
        if self.use_indices {
            gl.unmap_buffer(self.ibo);
            let (count, offset) = match &self.draw_range {
                None => (gl.get_buffer(self.ibo).unwrap().size() as i32, 0),
                Some(range) => ((range.end - range.start) as i32, range.start as i32),
            };
            if instance_count > 1 {
                gl.draw_elements_instanced(
                    self.draw_mode,
                    count,
                    INDEX_GL,
                    offset,
                    instance_count as i32,
                );
            } else {
                gl.draw_elements(self.draw_mode, count, INDEX_GL, offset);
            }
        } else {
            let (count, offset) = match &self.draw_range {
                None => (gl.get_buffer(self.vbo).unwrap().size() as i32, 0),
                Some(range) => ((range.end - range.start) as i32, range.start as i32),
            };
            if instance_count > 1 {
                gl.draw_arrays_instanced(self.draw_mode, offset, count, instance_count as i32);
            } else {
                gl.draw_arrays(self.draw_mode, offset, count);
            }
        }
    }
}

impl<T> Drop for Mesh<T> {
    fn drop(&mut self) {
        let mut gl = self.gl.borrow_mut();
        gl.destroy_buffer(self.vbo);
        gl.destroy_buffer(self.ibo);
    }
}
