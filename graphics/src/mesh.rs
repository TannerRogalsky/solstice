use super::{
    buffer::{Buffer, BufferType, Usage},
    vertex::Vertex,
    Context,
};

fn set_buffer<T>(buffer: &mut Buffer, data: &[T], offset: usize)
where
    T: Sized,
{
    buffer.write(
        unsafe {
            std::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                data.len() * std::mem::size_of::<T>(),
            )
        },
        offset * std::mem::size_of::<T>(),
    );
}

fn get_buffer<'a, T>(buffer: &'a Buffer) -> &'a [T]
where
    T: Sized,
{
    let data = buffer.memory_map();
    unsafe {
        // The lifetime of this slice is explicitly given because I'm not sure the compiler
        // can safely infer it in this unsafe function call. It might be fine but better safe
        // than sorry.
        std::slice::from_raw_parts::<'a, T>(
            data.as_ptr() as *const _,
            data.len() / std::mem::size_of::<T>(),
        )
    }
}

pub struct Mesh<V> {
    vbo: Buffer,
    draw_range: Option<std::ops::Range<usize>>,
    draw_mode: super::DrawMode,
    type_marker: std::marker::PhantomData<V>,
}

impl<V> Mesh<V>
where
    V: Vertex,
{
    pub fn new(gl: &mut Context, vertex_count: usize) -> Result<Self, super::GraphicsError> {
        let vbo = Buffer::new(
            gl,
            vertex_count * std::mem::size_of::<V>(),
            BufferType::Vertex,
            Usage::Dynamic,
        )?;
        Ok(Self {
            vbo,
            draw_range: None,
            draw_mode: super::DrawMode::Triangles,
            type_marker: std::marker::PhantomData,
        })
    }

    pub fn get_vertices(&self) -> &[V] {
        get_buffer(&self.vbo)
    }

    pub fn set_vertices(&mut self, vertices: &[V], offset: usize) {
        set_buffer(&mut self.vbo, vertices, offset);
    }

    pub fn set_draw_range(&mut self, draw_range: Option<std::ops::Range<usize>>) {
        self.draw_range = draw_range;
    }

    pub fn set_draw_mode(&mut self, draw_mode: super::DrawMode) {
        self.draw_mode = draw_mode;
    }

    pub fn attributes(&mut self) -> AttachedAttributes {
        AttachedAttributes {
            buffer: &mut self.vbo,
            formats: V::build_bindings(),
            step: 0,
            stride: std::mem::size_of::<V>(),
        }
    }

    pub fn draw(&mut self, gl: &mut Context) {
        self.draw_instanced(gl, 1);
    }

    pub fn draw_instanced(&mut self, gl: &mut Context, instance_count: usize) {
        self.internal_draw(gl, instance_count, &mut []);
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

        self.prepare_draw(gl, attached_attributes);

        let (count, offset) = match &self.draw_range {
            None => ((self.vbo.size() / std::mem::size_of::<V>()) as i32, 0),
            Some(range) => ((range.end - range.start) as i32, range.start as i32),
        };
        if instance_count > 1 {
            gl.draw_arrays_instanced(self.draw_mode, offset, count, instance_count as i32);
        } else {
            gl.draw_arrays(self.draw_mode, offset, count);
        }
    }

    fn prepare_draw(&mut self, gl: &mut Context, attached_attributes: &mut [AttachedAttributes]) {
        let stride = std::mem::size_of::<V>();

        gl.unmap_buffer(&mut self.vbo);
        for attr in attached_attributes.iter_mut() {
            gl.unmap_buffer(attr.buffer);
        }

        // there's likely a better way to accumulate all bindings into an easy to search collection
        let attached_bindings = V::build_bindings()
            .iter()
            .map(|binding| {
                (
                    binding,
                    stride,
                    0,
                    self.vbo.handle(),
                    self.vbo.buffer_type(),
                )
            })
            .chain(attached_attributes.iter().flat_map(|attributes| {
                attributes
                    .formats
                    .iter()
                    .map(|binding| {
                        (
                            binding,
                            attributes.stride,
                            attributes.step,
                            attributes.buffer.handle(),
                            attributes.buffer.buffer_type(),
                        )
                    })
                    .collect::<Vec<_>>()
            }))
            .collect::<Vec<_>>();

        let shader = gl
            .get_shader(gl.get_active_shader().expect("No active shader."))
            .unwrap();
        let mut desired_attribute_state = 0u32;
        let attributes = shader
            .attributes()
            .iter()
            .filter_map(|attr| {
                let binding = attached_bindings
                    .iter()
                    .find(|(binding, ..)| binding.name == attr.name.as_str());
                if binding.is_some() {
                    desired_attribute_state |= 1 << attr.location;
                }
                binding.cloned()
            })
            .collect::<Vec<_>>();
        gl.set_vertex_attributes(desired_attribute_state, &attributes);
    }
}

pub struct IndexedMesh<V, I> {
    mesh: Mesh<V>,
    ibo: Buffer,
    type_marker: std::marker::PhantomData<I>,
}

impl<V, I> IndexedMesh<V, I>
where
    V: Vertex,
    I: Index,
{
    pub fn new(
        gl: &mut Context,
        vertex_count: usize,
        index_count: usize,
    ) -> Result<Self, super::GraphicsError> {
        let ibo = Buffer::new(
            gl,
            index_count * std::mem::size_of::<I>(),
            BufferType::Index,
            Usage::Dynamic,
        )?;
        let mesh = Mesh::new(gl, vertex_count)?;
        Ok(Self {
            mesh,
            ibo,
            type_marker: std::marker::PhantomData,
        })
    }

    pub fn get_vertices(&self) -> &[V] {
        self.mesh.get_vertices()
    }

    pub fn get_indices(&self) -> &[I] {
        get_buffer(&self.ibo)
    }

    pub fn set_vertices(&mut self, vertices: &[V], offset: usize) {
        self.mesh.set_vertices(vertices, offset)
    }

    pub fn set_indices(&mut self, indices: &[I], offset: usize) {
        set_buffer(&mut self.ibo, indices, offset);
    }

    pub fn set_draw_range(&mut self, draw_range: Option<std::ops::Range<usize>>) {
        self.mesh.set_draw_range(draw_range)
    }

    pub fn set_draw_mode(&mut self, draw_mode: super::DrawMode) {
        self.mesh.set_draw_mode(draw_mode)
    }

    pub fn draw(&mut self, gl: &mut Context) {
        self.draw_instanced(gl, 1);
    }

    pub fn draw_instanced(&mut self, gl: &mut Context, instance_count: usize) {
        self.internal_draw(gl, instance_count, &mut []);
    }

    pub fn attributes(&mut self) -> AttachedAttributes {
        self.mesh.attributes()
    }

    fn internal_draw(
        &mut self,
        gl: &mut Context,
        instance_count: usize,
        attached_attributes: &mut [AttachedAttributes],
    ) {
        if let Some(draw_range) = &self.mesh.draw_range {
            if draw_range.start >= draw_range.end {
                return;
            }
        }

        self.mesh.prepare_draw(gl, attached_attributes);

        gl.unmap_buffer(&mut self.ibo);
        let (count, offset) = match &self.mesh.draw_range {
            None => ((self.ibo.size() / std::mem::size_of::<I>()) as i32, 0),
            Some(range) => ((range.end - range.start) as i32, range.start as i32),
        };
        if instance_count > 1 {
            gl.draw_elements_instanced(
                self.mesh.draw_mode,
                count,
                I::GL_TYPE,
                offset,
                instance_count as i32,
            );
        } else {
            gl.draw_elements(self.mesh.draw_mode, count, I::GL_TYPE, offset);
        }
    }
}

impl<V> HasAttributes for Mesh<V>
where
    V: Vertex,
{
    fn get_attributes(&mut self) -> AttachedAttributes {
        self.attributes()
    }
}

impl<V, I> HasAttributes for IndexedMesh<V, I>
where
    V: Vertex,
    I: Index,
{
    fn get_attributes(&mut self) -> AttachedAttributes {
        self.attributes()
    }
}

pub trait HasAttributes {
    fn get_attributes(&mut self) -> AttachedAttributes;
}

pub struct AttachedAttributes<'a> {
    buffer: &'a mut Buffer,
    formats: &'a [super::vertex::VertexFormat],
    step: u32,
    stride: usize,
}

pub struct MultiMesh<'a, V, I> {
    base: &'a mut IndexedMesh<V, I>,
    attachments: Vec<AttachedAttributes<'a>>,
}

impl<'a, V, I> MultiMesh<'a, V, I> {
    pub fn new(base: &'a mut IndexedMesh<V, I>, attachments: Vec<AttachedAttributes<'a>>) -> Self {
        Self { base, attachments }
    }
}

impl<'a, V, I> MultiMesh<'a, V, I>
where
    V: Vertex,
    I: Index,
{
    pub fn draw_instanced(&mut self, gl: &mut Context, instance_count: usize) {
        self.base
            .internal_draw(gl, instance_count, &mut self.attachments)
    }
}

pub trait MeshAttacher<'a, V, I>
where
    Self: Sized,
{
    fn attach<T>(self, other: &'a mut T) -> MultiMesh<'a, V, I>
    where
        T: HasAttributes,
    {
        Self::attach_with_step(self, other, 0)
    }

    fn attach_with_step<T>(self, other: &'a mut T, step: u32) -> MultiMesh<'a, V, I>
    where
        T: HasAttributes;
}

impl<'a, V, I> MeshAttacher<'a, V, I> for &'a mut IndexedMesh<V, I> {
    fn attach_with_step<T>(self, other: &'a mut T, step: u32) -> MultiMesh<'a, V, I>
    where
        T: HasAttributes,
    {
        let mut attachments = other.get_attributes();
        attachments.step = step;
        MultiMesh {
            base: self,
            attachments: vec![attachments],
        }
    }
}

impl<'a, V, I> MeshAttacher<'a, V, I> for MultiMesh<'a, V, I> {
    fn attach_with_step<T>(mut self, other: &'a mut T, step: u32) -> MultiMesh<'a, V, I>
    where
        T: HasAttributes,
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

pub trait Index {
    const GL_TYPE: u32;
}

impl Index for u32 {
    const GL_TYPE: u32 = glow::UNSIGNED_INT;
}

impl Index for u16 {
    const GL_TYPE: u32 = glow::UNSIGNED_SHORT;
}
