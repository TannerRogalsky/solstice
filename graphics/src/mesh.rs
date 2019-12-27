use super::{
    buffer::{Buffer, BufferType, Usage},
    vertex::Vertex,
    Context,
};

pub type Index = u32;
const INDEX_GL: u32 = glow::UNSIGNED_INT;

pub struct Mesh<T> {
    vbo: Buffer,
    ibo: Buffer,
    use_indices: bool,
    draw_range: Option<std::ops::Range<usize>>,
    vertex_marker: std::marker::PhantomData<T>,
    draw_mode: super::DrawMode,
}

impl<T> Mesh<T>
where
    T: Vertex,
{
    pub fn new(gl: &mut Context, size: usize) -> Self {
        Self::with_capacities(gl, size, size)
    }

    pub fn with_capacities(gl: &mut Context, vertex_count: usize, index_count: usize) -> Self {
        let vbo = Buffer::new(
            gl,
            vertex_count * std::mem::size_of::<T>(),
            BufferType::Vertex,
            Usage::Dynamic,
        );
        let ibo = Buffer::new(
            gl,
            index_count * std::mem::size_of::<Index>(),
            BufferType::Index,
            Usage::Dynamic,
        );
        Self {
            vbo,
            ibo,
            use_indices: false,
            draw_range: None,
            draw_mode: super::DrawMode::Triangles,
            vertex_marker: std::marker::PhantomData,
        }
    }

    fn set_buffer<V>(buffer: &mut Buffer, data: &[V], offset: usize)
    where
        V: Sized,
    {
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

    fn get_buffer<V>(buffer: &Buffer) -> &[V]
    where
        V: Sized,
    {
        let data = buffer.memory_map();
        unsafe {
            std::slice::from_raw_parts(
                data.as_ptr() as *const V,
                data.len() / std::mem::size_of::<V>(),
            )
        }
    }

    pub fn get_vertices(&self) -> &[T] {
        Self::get_buffer(&self.vbo)
    }

    pub fn get_indices(&self) -> &[Index] {
        Self::get_buffer(&self.ibo)
    }

    pub fn set_vertices(&mut self, vertices: &[T], offset: usize) {
        Self::set_buffer(&mut self.vbo, vertices, offset);
    }

    pub fn set_indices(&mut self, indices: &[Index], offset: usize) {
        self.use_indices = true;
        Self::set_buffer(&mut self.ibo, indices, offset);
    }

    pub fn set_draw_range(&mut self, draw_range: Option<std::ops::Range<usize>>) {
        self.draw_range = draw_range;
    }

    pub fn set_draw_mode(&mut self, draw_mode: super::DrawMode) {
        self.draw_mode = draw_mode;
    }

    pub fn draw(&mut self, gl: &mut Context) {
        self.draw_instanced(gl, 1);
    }

    pub fn draw_instanced(&mut self, gl: &mut Context, instance_count: usize) {
        if let Some(draw_range) = &self.draw_range {
            if draw_range.start >= draw_range.end {
                return;
            }
        }

        gl.bind_buffer(&self.vbo);

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
                vertex_format.map(|v| (v, &self.vbo))
            })
            .filter(Option::is_some)
            .map(Option::unwrap)
            .collect::<Vec<_>>();
        gl.set_vertex_attributes(desired_attribute_state, stride, &attributes);

        gl.unmap_buffer(&mut self.vbo);
        if self.use_indices {
            gl.unmap_buffer(&mut self.ibo);
            let (count, offset) = match &self.draw_range {
                None => ((self.ibo.size() / std::mem::size_of::<Index>()) as i32, 0),
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
                None => ((self.vbo.size() / std::mem::size_of::<Index>()) as i32, 0),
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
