use super::{
    buffer::{Buffer, BufferType, MappedBuffer, Usage},
    vertex::{Vertex, VertexFormat},
    Context,
};

// The lifetime of this slice is explicitly given because I'm not sure the compiler
// can safely infer it in this unsafe function call. It might be fine but better safe
// than sorry.
fn to_bytes<'a, V>(data: &'a [V]) -> &'a [u8]
where
    V: Sized,
{
    unsafe {
        std::slice::from_raw_parts::<'a, u8>(
            data.as_ptr() as *const _,
            data.len() * std::mem::size_of::<V>(),
        )
    }
}

fn from_bytes<'a, V>(data: &'a [u8]) -> &'a [V] where V: Sized {
    unsafe {
        std::slice::from_raw_parts::<'a, V>(
            data.as_ptr() as *const _,
            data.len() / std::mem::size_of::<V>(),
        )
    }
}

fn set_buffer<T>(buffer: &mut MappedBuffer, data: &[T], offset: usize)
where
    T: Sized,
{
    buffer.write(to_bytes(data), offset * std::mem::size_of::<T>());
}

fn get_buffer<'a, T>(buffer: &'a MappedBuffer) -> &'a [T]
where
    T: Sized,
{
    from_bytes(buffer.memory_map())
}

pub type BindingInfo<'a> = (&'a VertexFormat, usize, u32, super::BufferKey, BufferType);

/// The Mesh represents a set of vertices to be drawn by a shader program and how to draw them.
/// A well-formed Vertex implementation will provide information to the mesh about it's layout in
/// order to properly sync the data to the GPU. A derive macro exists in
/// [`Vertex`](graphics_macro::Vertex) that will derive this implementation for you.
///
/// ```
/// use graphics::vertex::{VertexFormat, AttributeType};
/// #[repr(C, packed)]
/// struct TestVertex {
///     position: [f32; 2],
///     color: [f32; 4],
/// }
///
/// impl graphics::vertex::Vertex for TestVertex {
///     fn build_bindings() -> &'static [VertexFormat] {
///         &[VertexFormat {
///             name: "position",
///             offset: 0,
///             atype: AttributeType::F32F32,
///             normalize: false,
///         }, VertexFormat {
///             name: "color",
///             offset: std::mem::size_of::<[f32; 2]>(),
///             atype: AttributeType::F32F32F32F32,
///             normalize: false
///         }]
///     }
/// }
///
/// let vertex_data = vec![
///     TestVertex {
///         position: [0.5, 0.],
///         color: [1., 0., 0., 1.]
///     },
///     TestVertex {
///         position: [0., 1.0],
///         color: [0., 1., 0., 1.]
///     },
///     TestVertex {
///         position: [1., 0.],
///         color: [0., 0., 1., 1.]
///     },
/// ];
/// ```
///
/// Vertex data is then copied into the mesh.
///
/// ```ignore
/// let mut mesh = graphics::mesh::Mesh::new(&mut ctx, 3);
/// mesh.set_vertices(&vertex_data, 0);
/// ```
///
/// Once constructed, a Mesh is of an immutable size but the draw range can be modified to
/// effectively change it's size without changing the underlying memory's size.
///
/// ```ignore
/// let mut mesh = graphics::mesh::Mesh::new(&mut ctx, 3000).unwrap();
/// mesh.set_draw_range(Some(0..3)); // draws only the first three vertices of the 3000 allocated
/// ```
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
    /// Construct a Mesh with a given number of vertices.
    pub fn new(ctx: &mut Context, size: usize) -> Result<Self, super::GraphicsError> {
        let vbo = Buffer::new(
            ctx,
            size * std::mem::size_of::<V>(),
            BufferType::Vertex,
            Usage::Dynamic,
        )?;
        Ok(Self::with_buffer(vbo))
    }

    pub fn with_data(ctx: &mut Context, vertices: &[V]) -> Result<Self, super::GraphicsError> {
        let vbo = Buffer::with_data(ctx, to_bytes(vertices), BufferType::Vertex, Usage::Dynamic)?;
        Ok(Self::with_buffer(vbo))
    }

    pub fn with_buffer(buffer: Buffer) -> Self {
        Self {
            vbo: buffer,
            draw_range: None,
            draw_mode: super::DrawMode::Triangles,
            type_marker: std::marker::PhantomData,
        }
    }

    /// Write new data into a range of the Mesh's vertex data.
    pub fn set_vertices(&self, ctx: &mut super::Context, vertices: &[V], offset: usize) {
        ctx.buffer_static_draw(
            &self.vbo,
            to_bytes(vertices),
            offset * std::mem::size_of::<V>(),
        )
    }

    /// Sets the range of vertices to be drawn. Passing `None` will draw the entire mesh.
    pub fn set_draw_range(&mut self, draw_range: Option<std::ops::Range<usize>>) {
        self.draw_range = draw_range;
    }

    /// Sets how the vertex data should be tesselated.
    pub fn set_draw_mode(&mut self, draw_mode: super::DrawMode) {
        self.draw_mode = draw_mode;
    }

    /// Draw the mesh.
    pub fn draw(&self, gl: &mut Context) {
        self.draw_instanced(gl, 1);
    }

    pub fn attributes(&self) -> AttachedAttributes {
        AttachedAttributes {
            buffer: &self.vbo,
            formats: V::build_bindings(),
            step: 0,
            stride: std::mem::size_of::<V>(),
        }
    }

    pub fn draw_range(&self) -> std::ops::Range<usize> {
        self.draw_range
            .clone()
            .unwrap_or(0..(self.vbo.size() / std::mem::size_of::<V>()))
    }

    pub fn draw_mode(&self) -> super::DrawMode {
        self.draw_mode
    }

    /// Draw the mesh multiple times using the same vertex data. This might be useful if you're
    /// using per instance data from a uniform or a separate [attached mesh](graphics::mesh::MeshAttacher).
    pub fn draw_instanced(&self, ctx: &mut Context, instance_count: usize) {
        self.internal_draw(ctx, instance_count, &[]);
    }

    fn internal_draw(
        &self,
        ctx: &mut Context,
        instance_count: usize,
        attached_attributes: &[AttachedAttributes],
    ) {
        let draw_range = self.draw_range();
        if draw_range.start >= draw_range.end {
            return;
        }

        ctx.bind_buffer(self.vbo.handle(), self.vbo.buffer_type());
        self.prepare_draw(ctx, attached_attributes);

        let (count, offset) = (
            (draw_range.end - draw_range.start) as i32,
            draw_range.start as i32,
        );
        if instance_count > 1 {
            ctx.draw_arrays_instanced(self.draw_mode(), offset, count, instance_count as i32);
        } else {
            ctx.draw_arrays(self.draw_mode(), offset, count);
        }
    }

    fn prepare_draw(&self, ctx: &mut Context, attached_attributes: &[AttachedAttributes]) {
        let AttachedAttributes {
            buffer,
            formats,
            step,
            stride,
        } = self.attributes();

        // gl.unmap_buffer(&mut self.vbo);
        // for attr in attached_attributes.iter_mut() {
        //     gl.unmap_buffer(attr.buffer);
        // }

        // there's likely a better way to accumulate all bindings into an easy to search collection
        let attached_bindings = formats
            .iter()
            .map(|binding| (binding, stride, step, buffer.handle(), buffer.buffer_type()))
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

        let shader = ctx.get_active_shader().expect("No active shader.");
        let mut desired_attribute_state = 0u32;
        let mut attributes = [None; 32];
        for attr in shader.attributes().iter() {
            let binding = attached_bindings
                .iter()
                .find(|(binding, ..)| binding.name == attr.name.as_str())
                .cloned();
            if let Some(binding) = binding {
                desired_attribute_state |= 1 << attr.location;
                attributes[attr.location as usize] = Some(binding);
            }
        }
        ctx.set_vertex_attributes(desired_attribute_state, &attributes);
    }
}

pub struct MappedMesh<V> {
    inner: Mesh<V>,
    memory_map: MappedBuffer,
}

impl<V> MappedMesh<V>
where
    V: Vertex,
{
    pub fn new(ctx: &mut super::Context, size: usize) -> Result<Self, super::GraphicsError> {
        let inner = Mesh::new(ctx, size)?;
        let memory_map =
            MappedBuffer::with_shape(inner.vbo.clone(), [size * std::mem::size_of::<V>()]);
        Ok(Self { inner, memory_map })
    }

    pub fn set_vertices(&mut self, vertices: &[V], offset: usize) {
        set_buffer(&mut self.memory_map, vertices, offset)
    }

    pub fn get_vertices(&self) -> &[V] {
        get_buffer(&self.memory_map)
    }

    pub fn unmap(&mut self, ctx: &mut super::Context) -> &Mesh<V> {
        self.memory_map.unmap(ctx);
        &self.inner
    }
}

/// A mesh with vertex data that is indexed with separate data.
///
/// This is useful if you have a number of vertices that you would otherwise have to duplicate
/// because indices are generally smaller than a vertex so duplicating them is more performant.
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
    /// Construct a mesh with a given number of vertices and indices.
    pub fn new(
        ctx: &mut Context,
        vertex_count: usize,
        index_count: usize,
    ) -> Result<Self, super::GraphicsError> {
        let ibo = Buffer::new(
            ctx,
            index_count * std::mem::size_of::<I>(),
            BufferType::Index,
            Usage::Dynamic,
        )?;
        let mesh = Mesh::new(ctx, vertex_count)?;
        Ok(Self {
            mesh,
            ibo,
            type_marker: std::marker::PhantomData,
        })
    }

    pub fn with_data(
        ctx: &mut Context,
        vertices: &[V],
        indices: &[I],
    ) -> Result<Self, super::GraphicsError> {
        let ibo = Buffer::with_data(ctx, to_bytes(indices), BufferType::Index, Usage::Dynamic)?;
        let mesh = Mesh::with_data(ctx, vertices)?;
        Ok(Self {
            mesh,
            ibo,
            type_marker: std::marker::PhantomData,
        })
    }

    /// Construct an indexed mesh from a non-indexed mesh.
    pub fn with_mesh(
        ctx: &mut Context,
        mesh: Mesh<V>,
        index_count: usize,
    ) -> Result<Self, super::GraphicsError> {
        let ibo = Buffer::new(
            ctx,
            index_count * std::mem::size_of::<I>(),
            BufferType::Index,
            mesh.vbo.usage(),
        )?;
        Ok(Self {
            mesh,
            ibo,
            type_marker: std::marker::PhantomData,
        })
    }

    /// Write new data into a range of the Mesh's vertex data.
    pub fn set_vertices(&self, ctx: &mut Context, vertices: &[V], offset: usize) {
        ctx.buffer_static_draw(
            &self.mesh.vbo,
            to_bytes(vertices),
            offset * std::mem::size_of::<V>(),
        )
    }

    /// Write new data into a range of the Mesh's vertex data.
    pub fn set_indices(&self, ctx: &mut Context, indices: &[I], offset: usize) {
        ctx.buffer_static_draw(
            &self.ibo,
            to_bytes(indices),
            offset * std::mem::size_of::<I>(),
        )
    }

    pub fn set_draw_range(&mut self, draw_range: Option<std::ops::Range<usize>>) {
        self.mesh.set_draw_range(draw_range)
    }

    pub fn set_draw_mode(&mut self, draw_mode: super::DrawMode) {
        self.mesh.set_draw_mode(draw_mode)
    }

    pub fn draw(&self, gl: &mut Context) {
        self.draw_instanced(gl, 1);
    }

    pub fn draw_instanced(&self, gl: &mut Context, instance_count: usize) {
        self.internal_draw(gl, instance_count, &[]);
    }

    pub fn attributes(&self) -> AttachedAttributes {
        self.mesh.attributes()
    }

    fn internal_draw(
        &self,
        ctx: &mut Context,
        instance_count: usize,
        attached_attributes: &[AttachedAttributes],
    ) {
        if let Some(draw_range) = &self.mesh.draw_range {
            if draw_range.start >= draw_range.end {
                return;
            }
        }

        ctx.bind_buffer(self.mesh.vbo.handle(), self.mesh.vbo.buffer_type());
        ctx.bind_buffer(self.ibo.handle(), self.ibo.buffer_type());
        self.mesh.prepare_draw(ctx, attached_attributes);

        let (count, offset) = match &self.mesh.draw_range {
            None => ((self.ibo.size() / std::mem::size_of::<I>()) as i32, 0),
            Some(range) => ((range.end - range.start) as i32, range.start as i32),
        };
        if instance_count > 1 {
            ctx.draw_elements_instanced(
                self.mesh.draw_mode,
                count,
                I::GL_TYPE,
                offset,
                instance_count as i32,
            );
        } else {
            ctx.draw_elements(self.mesh.draw_mode, count, I::GL_TYPE, offset);
        }
    }
}

pub struct MappedIndexedMesh<V, I> {
    inner: IndexedMesh<V, I>,
    vbo: MappedBuffer,
    ibo: MappedBuffer,
}

impl<V, I> MappedIndexedMesh<V, I>
where
    V: Vertex,
    I: Index,
{
    pub fn new(
        gl: &mut Context,
        vertex_count: usize,
        index_count: usize,
    ) -> Result<Self, super::GraphicsError> {
        let inner = IndexedMesh::new(gl, vertex_count, index_count)?;
        let vbo = MappedBuffer::with_shape(inner.mesh.vbo.clone(), inner.mesh.vbo.size());
        let ibo = MappedBuffer::with_shape(inner.ibo.clone(), inner.ibo.size());
        Ok(Self { inner, vbo, ibo })
    }

    pub fn set_vertices(&mut self, vertices: &[V], offset: usize) {
        set_buffer(&mut self.vbo, vertices, offset)
    }

    pub fn get_vertices(&self) -> &[V] {
        get_buffer(&self.vbo)
    }

    pub fn set_indices(&mut self, indices: &[I], offset: usize) {
        set_buffer(&mut self.ibo, indices, offset)
    }

    pub fn get_indices(&self) -> &[I] {
        get_buffer(&self.ibo)
    }

    pub fn unmap(&mut self, ctx: &mut Context) -> &IndexedMesh<V, I> {
        self.vbo.unmap(ctx);
        self.ibo.unmap(ctx);
        &self.inner
    }
}

pub struct AttachedAttributes<'a> {
    buffer: &'a Buffer,
    formats: &'a [VertexFormat],
    step: u32,
    stride: usize,
}

// TODO: Redo this but without the trait: implement behaviour for specific structs
// pub struct MultiMesh<'a, T> {
//     base: &'a T,
//     attachments: Vec<AttachedAttributes<'a>>,
// }
//
// impl<'a, T> MultiMesh<'a, T> {
//     pub fn new(base: &'a T, attachments: Vec<AttachedAttributes<'a>>) -> Self {
//         Self { base, attachments }
//     }
// }
//
// impl<'a, T> MultiMesh<'a, T>
// where
//     T: MeshTrait,
// {
//     pub fn draw_instanced(&mut self, gl: &mut Context, instance_count: usize) {
//         self.base
//             .secret_draw(gl, instance_count, &mut self.attachments)
//     }
// }
//
// pub trait MeshAttacher<'a, B>
// where
//     Self: Sized,
// {
//     fn attach<T>(self, other: &'a mut T) -> MultiMesh<'a, B>
//     where
//         T: MeshTrait,
//     {
//         Self::attach_with_step(self, other, 0)
//     }
//
//     fn attach_with_step<T>(self, other: &'a mut T, step: u32) -> MultiMesh<'a, B>
//     where
//         T: MeshTrait;
// }
//
// impl<'a, S> MeshAttacher<'a, S> for &'a mut S {
//     fn attach_with_step<T>(self, other: &'a mut T, step: u32) -> MultiMesh<'a, S>
//     where
//         T: MeshTrait,
//     {
//         let mut attachments = other.get_attributes();
//         attachments.step = step;
//         MultiMesh {
//             base: self,
//             attachments: vec![attachments],
//         }
//     }
// }
//
// impl<'a, B> MeshAttacher<'a, B> for MultiMesh<'a, B> {
//     fn attach_with_step<T>(mut self, other: &'a mut T, step: u32) -> MultiMesh<'a, B>
//     where
//         T: MeshTrait,
//     {
//         let mut attachments = other.get_attributes();
//         attachments.step = step;
//         self.attachments.push(attachments);
//         MultiMesh {
//             base: self.base,
//             attachments: self.attachments,
//         }
//     }
// }

pub trait Index {
    const GL_TYPE: u32;
}

impl Index for u32 {
    const GL_TYPE: u32 = glow::UNSIGNED_INT;
}

impl Index for u16 {
    const GL_TYPE: u32 = glow::UNSIGNED_SHORT;
}
