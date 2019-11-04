use super::{
    buffer::{BufferType, Usage},
    vertex::Vertex,
    BufferKey, Context,
};
use std::cell::RefCell;
use std::rc::Rc;

pub type Index = u32;
const INDEX_GL: u32 = glow::UNSIGNED_INT;

pub struct Mesh<T> {
    gl: Rc<RefCell<Context>>,
    vbo: BufferKey,
    ibo: BufferKey,
    vertex_count: usize,
    index_count: usize,
    draw_range: Option<std::ops::Range<usize>>,
    vertex_marker: std::marker::PhantomData<T>,
}

impl<T> Mesh<T>
where
    T: Vertex,
{
    pub fn new(gl: Rc<RefCell<Context>>, size: usize) -> Self {
        let vbo = gl.borrow_mut().new_buffer(
            size * std::mem::size_of::<T>(),
            BufferType::Vertex,
            Usage::Stream,
        );
        let ibo = gl.borrow_mut().new_buffer(
            size * std::mem::size_of::<Index>(),
            BufferType::Index,
            Usage::Static,
        );
        Self {
            gl,
            vbo,
            ibo,
            vertex_count: 0,
            index_count: 0,
            draw_range: None,
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
        self.vertex_count = vertices.len();
        self.set_buffer(self.vbo, vertices, offset);
    }

    pub fn set_indices(&mut self, indices: &[Index], offset: usize) {
        self.index_count = indices.len();
        self.set_buffer(self.ibo, indices, offset);
    }

    pub fn set_draw_range(&mut self, draw_range: Option<std::ops::Range<usize>>) {
        self.draw_range = draw_range;
    }

    pub fn draw(&mut self) {
        let mut gl = self.gl.borrow_mut();
        gl.bind_buffer(self.vbo);

        let stride = std::mem::size_of::<T>();
        let bindings = T::build_bindings();

        let shader = gl.get_shader(gl.get_active_shader().unwrap()).unwrap();
        let attributes = shader
            .attributes()
            .iter()
            .map(|attr| {
                (
                    bindings
                        .iter()
                        .find(|&binding| binding.name == attr.name.as_str())
                        .unwrap(),
                    self.vbo,
                )
            })
            .collect::<Vec<(&super::vertex::VertexFormat, super::BufferKey)>>();
        let desired = shader
            .attributes()
            .iter()
            .fold(0u32, |acc, attr| acc | (1 << attr.location));
        gl.set_vertex_attributes(desired, stride, &attributes);

        gl.unmap_buffer(self.vbo);
        gl.unmap_buffer(self.ibo);
        let (count, offset) = match &self.draw_range {
            None => (self.index_count as i32, 0),
            Some(range) => ((range.end - range.start) as i32, range.start as i32),
        };
        gl.draw_elements(glow::TRIANGLES, count, INDEX_GL, offset);
    }
}

impl<T> Drop for Mesh<T> {
    fn drop(&mut self) {
        let mut gl = self.gl.borrow_mut();
        gl.destroy_buffer(self.vbo);
        gl.destroy_buffer(self.ibo);
    }
}
