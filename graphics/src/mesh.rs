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

    fn get_buffer<'a, V>(buffer: &'a Buffer) -> &'a [V]
    where
        V: Sized,
    {
        let data = buffer.memory_map();
        unsafe {
            // The lifetime of this slice is explicitly given because I'm not sure the compiler
            // can safely infer it in this unsafe function call. It might be fine but better safe
            // than sorry.
            std::slice::from_raw_parts::<'a, V>(
                data.as_ptr() as *const _,
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
        self.internal_draw(gl, instance_count, &mut []);
    }

    pub fn get_attributes(&mut self) -> AttachedAttributes {
        AttachedAttributes {
            buffer: &mut self.vbo,
            formats: T::build_bindings(),
            step: 0,
            stride: std::mem::size_of::<T>(),
        }
    }

    fn internal_draw(
        &mut self,
        gl: &mut Context,
        instance_count: usize,
        attached_attributes: &mut [AttachedAttributes],
    ) {
        if let Some(draw_range) = &self.draw_range {
            if draw_range.start >= draw_range.end {
                return;
            }
        }

        let stride = std::mem::size_of::<T>();

        gl.unmap_buffer(&mut self.vbo);
        for attr in attached_attributes.iter_mut() {
            gl.unmap_buffer(attr.buffer);
        }

        let shader = gl
            .get_shader(gl.get_active_shader().expect("No active shader."))
            .unwrap();
        let mut desired_attribute_state = 0u32;
        let attributes = shader
            .attributes()
            .iter()
            .filter_map(|attr| {
                // for each attribute in the shader, try to map it to one of our attachements
                // do self's bindings first and then try the attached attributes
                // TODO: it would be great if this were less tiered and simpler
                T::build_bindings()
                    .iter()
                    .find(|binding| binding.name == attr.name.as_str())
                    .map(|binding| {
                        desired_attribute_state |= 1 << attr.location;
                        (
                            binding,
                            stride,
                            0,
                            self.vbo.handle(),
                            self.vbo.buffer_type(),
                        )
                    })
                    .or_else(|| {
                        attached_attributes
                            .iter()
                            .find_map(|attributes| {
                                attributes
                                    .formats
                                    .iter()
                                    .find(|binding| binding.name == attr.name.as_str())
                                    .map(|binding| {
                                        (
                                            binding,
                                            attributes.stride,
                                            attributes.step,
                                            attributes.buffer.handle(),
                                            attributes.buffer.buffer_type(),
                                        )
                                    })
                            })
                            .map(|(format, stride, step, buffer_key, buffer_type)| {
                                desired_attribute_state |= 1 << attr.location;
                                (format, stride, step, buffer_key, buffer_type)
                            })
                    })
            })
            .collect::<Vec<_>>();
        gl.set_vertex_attributes(desired_attribute_state, &attributes);

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

pub struct AttachedAttributes<'a> {
    buffer: &'a mut Buffer,
    formats: &'a [super::vertex::VertexFormat],
    step: u32,
    stride: usize,
}

pub struct MultiMesh<'a, T> {
    base: &'a mut Mesh<T>,
    attachments: Vec<AttachedAttributes<'a>>,
}

impl<'a, T> MultiMesh<'a, T> {
    pub fn new(base: &'a mut Mesh<T>, attachments: Vec<AttachedAttributes<'a>>) -> Self {
        Self { base, attachments }
    }
}

impl<'a, T> MultiMesh<'a, T>
where
    T: Vertex,
{
    pub fn draw_instanced(&mut self, gl: &mut Context, instance_count: usize) {
        self.base
            .internal_draw(gl, instance_count, &mut self.attachments)
    }
}

pub trait MeshAttacher<'a, T>
where
    Self: Sized,
{
    fn attach<N>(self, other: &'a mut Mesh<N>) -> MultiMesh<'a, T>
    where
        N: Vertex,
    {
        Self::attach_with_step(self, other, 0)
    }

    fn attach_with_step<N>(self, other: &'a mut Mesh<N>, step: u32) -> MultiMesh<'a, T>
    where
        N: Vertex;
}

impl<'a, T> MeshAttacher<'a, T> for &'a mut Mesh<T> {
    fn attach_with_step<N>(self, other: &'a mut Mesh<N>, step: u32) -> MultiMesh<'a, T>
    where
        N: Vertex,
    {
        let mut attachments = other.get_attributes();
        attachments.step = step;
        MultiMesh {
            base: self,
            attachments: vec![attachments],
        }
    }
}

impl<'a, T> MeshAttacher<'a, T> for MultiMesh<'a, T> {
    fn attach_with_step<N>(mut self, other: &'a mut Mesh<N>, step: u32) -> MultiMesh<'a, T>
    where
        N: Vertex,
    {
        let mut attachments = other.get_attributes();
        attachments.step = step;
        self.attachments.push(attachments);
        MultiMesh {
            base: self.base,
            attachments: self.attachments,
        }
    }
}
