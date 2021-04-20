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

fn from_bytes<'a, V>(data: &'a [u8]) -> &'a [V]
where
    V: Sized,
{
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

fn get_buffer<T>(buffer: &MappedBuffer) -> &[T]
where
    T: Sized,
{
    from_bytes(buffer.memory_map())
}

pub type BindingInfo<'a> = (&'a VertexFormat, usize, u32, super::BufferKey, BufferType);

/// The Mesh represents a set of vertices to be drawn by a shader program and how to draw them.
/// A well-formed Vertex implementation will provide information to the mesh about it's layout in
/// order to properly sync the data to the GPU. A derive macro exists in
/// [`Vertex`](solstice_derive::Vertex) that will derive this implementation for you.
///
/// ```
/// use solstice::vertex::{VertexFormat, AttributeType};
/// #[repr(C, packed)]
/// struct TestVertex {
///     position: [f32; 2],
///     color: [f32; 4],
/// }
///
/// impl solstice::vertex::Vertex for TestVertex {
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
/// let mut mesh = solstice::mesh::Mesh::new(&mut ctx, 3);
/// mesh.set_vertices(&vertex_data, 0);
/// ```
///
/// Once constructed, a Mesh is of an immutable size but the draw range can be modified to
/// effectively change it's size without changing the underlying memory's size.
///
/// ```ignore
/// let mut mesh = solstice::mesh::Mesh::new(&mut ctx, 3000).unwrap();
/// mesh.set_draw_range(Some(0..3)); // draws only the first three vertices of the 3000 allocated
/// ```
#[derive(Debug, PartialEq)]
pub struct VertexMesh<V> {
    vbo: Buffer,
    draw_range: Option<std::ops::Range<usize>>,
    draw_mode: super::DrawMode,
    type_marker: std::marker::PhantomData<V>,
}

impl<V> VertexMesh<V>
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
        ctx.bind_buffer(self.vbo.handle(), self.vbo.buffer_type());
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

    pub fn draw_range(&self) -> std::ops::Range<usize> {
        self.draw_range.clone().unwrap_or(0..(self.len()))
    }

    pub fn draw_mode(&self) -> super::DrawMode {
        self.draw_mode
    }

    pub fn len(&self) -> usize {
        self.vbo.size() / std::mem::size_of::<V>()
    }
}

#[derive(Debug, PartialEq)]
pub struct MappedVertexMesh<V> {
    inner: VertexMesh<V>,
    memory_map: MappedBuffer,
}

impl<V> MappedVertexMesh<V>
where
    V: Vertex,
{
    pub fn new(ctx: &mut super::Context, size: usize) -> Result<Self, super::GraphicsError> {
        let inner = VertexMesh::new(ctx, size)?;
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

    pub fn unmap(&mut self, ctx: &mut super::Context) -> &VertexMesh<V> {
        self.memory_map.unmap(ctx);
        &self.inner
    }
}

/// A mesh with vertex data that is indexed with separate data.
///
/// This is useful if you have a number of vertices that you would otherwise have to duplicate
/// because indices are generally smaller than a vertex so duplicating them is more performant.
#[derive(Debug, PartialEq)]
pub struct IndexedMesh<V, I> {
    mesh: VertexMesh<V>,
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
        let mesh = VertexMesh::new(ctx, vertex_count)?;
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
        let mesh = VertexMesh::with_data(ctx, vertices)?;
        Ok(Self {
            mesh,
            ibo,
            type_marker: std::marker::PhantomData,
        })
    }

    /// Construct an indexed mesh from a non-indexed mesh.
    pub fn with_mesh(
        ctx: &mut Context,
        mesh: VertexMesh<V>,
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
        self.mesh.set_vertices(ctx, vertices, offset)
    }

    /// Write new data into a range of the Mesh's vertex data.
    pub fn set_indices(&self, ctx: &mut Context, indices: &[I], offset: usize) {
        ctx.bind_buffer(self.ibo.handle(), self.ibo.buffer_type());
        ctx.buffer_static_draw(
            &self.ibo,
            to_bytes(indices),
            offset * std::mem::size_of::<I>(),
        )
    }

    pub fn set_draw_range(&mut self, draw_range: Option<std::ops::Range<usize>>) {
        self.mesh.set_draw_range(draw_range)
    }

    pub fn draw_range(&self) -> std::ops::Range<usize> {
        self.mesh
            .draw_range
            .clone()
            .unwrap_or(0..(self.ibo.size() / std::mem::size_of::<I>()))
    }

    pub fn set_draw_mode(&mut self, draw_mode: super::DrawMode) {
        self.mesh.set_draw_mode(draw_mode)
    }
}

#[derive(Debug, PartialEq)]
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

    pub fn with_data(
        ctx: &mut Context,
        vertices: Vec<V>,
        indices: Vec<I>,
    ) -> Result<Self, super::GraphicsError> {
        let inner = IndexedMesh::with_data(ctx, &vertices, &indices)?;
        let vbo = MappedBuffer::from_vec(inner.mesh.vbo.clone(), unsafe {
            let mut vertices = std::mem::ManuallyDrop::new(vertices);
            Vec::from_raw_parts(
                vertices.as_mut_ptr() as *mut _,
                vertices.len() * std::mem::size_of::<V>(),
                vertices.capacity() * std::mem::size_of::<V>(),
            )
        });
        let ibo = MappedBuffer::from_vec(inner.ibo.clone(), unsafe {
            let mut indices = std::mem::ManuallyDrop::new(indices);
            Vec::from_raw_parts(
                indices.as_mut_ptr() as *mut _,
                indices.len() * std::mem::size_of::<I>(),
                indices.capacity() * std::mem::size_of::<I>(),
            )
        });
        Ok(Self { inner, vbo, ibo })
    }

    pub fn vertex_capacity(&self) -> usize {
        self.vbo.memory_map.len() / std::mem::size_of::<V>()
    }

    pub fn index_capacity(&self) -> usize {
        self.ibo.memory_map.len() / std::mem::size_of::<I>()
    }

    pub fn set_draw_range(&mut self, draw_range: Option<std::ops::Range<usize>>) {
        self.inner.set_draw_range(draw_range)
    }

    pub fn draw_range(&self) -> std::ops::Range<usize> {
        self.inner.draw_range()
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

#[derive(Clone, Debug)]
pub struct AttachedAttributes<'a> {
    pub buffer: &'a Buffer,
    pub formats: &'a [VertexFormat],
    pub step: u32,
    pub stride: usize,
}

pub trait Mesh {
    fn attachments(&self) -> Vec<AttachedAttributes>;
    fn draw(
        &self,
        ctx: &mut super::Context,
        draw_range: std::ops::Range<usize>,
        draw_mode: super::DrawMode,
        instance_count: usize,
    );
}

impl<V: Vertex> Mesh for VertexMesh<V> {
    fn attachments(&self) -> Vec<AttachedAttributes> {
        vec![AttachedAttributes {
            buffer: &self.vbo,
            formats: V::build_bindings(),
            step: 0,
            stride: std::mem::size_of::<V>(),
        }]
    }

    fn draw(
        &self,
        ctx: &mut super::Context,
        draw_range: std::ops::Range<usize>,
        draw_mode: super::DrawMode,
        instance_count: usize,
    ) {
        if draw_range.start >= draw_range.end {
            return;
        }

        let (count, offset) = (
            (draw_range.end - draw_range.start) as i32,
            draw_range.start as i32,
        );
        if instance_count > 1 {
            ctx.draw_arrays_instanced(draw_mode, offset, count, instance_count as i32);
        } else {
            ctx.draw_arrays(draw_mode, offset, count);
        }
    }
}

impl<V: Vertex> Mesh for &VertexMesh<V> {
    fn attachments(&self) -> Vec<AttachedAttributes> {
        VertexMesh::attachments(self)
    }

    fn draw(
        &self,
        ctx: &mut super::Context,
        draw_range: std::ops::Range<usize>,
        draw_mode: super::DrawMode,
        instance_count: usize,
    ) {
        VertexMesh::draw(self, ctx, draw_range, draw_mode, instance_count)
    }
}

impl<V: Vertex, I: Index> Mesh for IndexedMesh<V, I> {
    fn attachments(&self) -> Vec<AttachedAttributes> {
        self.mesh.attachments()
    }

    fn draw(
        &self,
        ctx: &mut super::Context,
        draw_range: std::ops::Range<usize>,
        draw_mode: super::DrawMode,
        instance_count: usize,
    ) {
        if draw_range.start >= draw_range.end {
            return;
        }

        let (count, offset) = (
            (draw_range.end - draw_range.start) as i32,
            draw_range.start as i32,
        );

        let ibo = &self.ibo;
        ctx.bind_buffer(ibo.handle(), ibo.buffer_type());
        if instance_count > 1 {
            ctx.draw_elements_instanced(
                draw_mode,
                count,
                I::GL_TYPE,
                offset,
                instance_count as i32,
            );
        } else {
            ctx.draw_elements(draw_mode, count, I::GL_TYPE, offset);
        }
    }
}

impl<V: Vertex, I: Index> Mesh for &IndexedMesh<V, I> {
    fn attachments(&self) -> Vec<AttachedAttributes> {
        IndexedMesh::attachments(self)
    }

    fn draw(
        &self,
        ctx: &mut super::Context,
        draw_range: std::ops::Range<usize>,
        draw_mode: super::DrawMode,
        instance_count: usize,
    ) {
        IndexedMesh::draw(self, ctx, draw_range, draw_mode, instance_count)
    }
}

// TODO: Redo this but without the trait: implement behaviour for specific structs
pub struct MultiMesh<'a> {
    ibo: Option<(&'a Buffer, u32)>,
    attachments: Vec<AttachedAttributes<'a>>,
}

impl<'a> Mesh for MultiMesh<'a> {
    fn attachments(&self) -> Vec<AttachedAttributes> {
        self.attachments.clone()
    }

    fn draw(
        &self,
        ctx: &mut Context,
        draw_range: std::ops::Range<usize>,
        draw_mode: super::DrawMode,
        instance_count: usize,
    ) {
        match self.ibo {
            None => {
                if draw_range.start >= draw_range.end {
                    return;
                }

                let (count, offset) = (
                    (draw_range.end - draw_range.start) as i32,
                    draw_range.start as i32,
                );
                if instance_count > 1 {
                    ctx.draw_arrays_instanced(draw_mode, offset, count, instance_count as i32);
                } else {
                    ctx.draw_arrays(draw_mode, offset, count);
                }
            }
            Some((ibo, element_type)) => {
                if draw_range.start >= draw_range.end {
                    return;
                }

                let (count, offset) = (
                    (draw_range.end - draw_range.start) as i32,
                    draw_range.start as i32,
                );

                ctx.bind_buffer(ibo.handle(), ibo.buffer_type());
                if instance_count > 1 {
                    ctx.draw_elements_instanced(
                        draw_mode,
                        count,
                        element_type,
                        offset,
                        instance_count as i32,
                    );
                } else {
                    ctx.draw_elements(draw_mode, count, element_type, offset);
                }
            }
        }
    }
}

pub trait MeshAttacher: Mesh {
    fn attach<'a, T: Mesh>(&'a self, other: &'a T) -> MultiMesh<'a> {
        Self::attach_with_step(self, other, 0)
    }

    fn attach_with_step<'a, T: Mesh>(&'a self, other: &'a T, step: u32) -> MultiMesh<'a>;
}

impl<V: Vertex> MeshAttacher for VertexMesh<V> {
    fn attach_with_step<'a, T: Mesh>(&'a self, other: &'a T, step: u32) -> MultiMesh<'a> {
        let mut attachments = self.attachments();
        attachments.extend(other.attachments().into_iter().map(|mut a| {
            a.step = step;
            a
        }));
        MultiMesh {
            ibo: None,
            attachments,
        }
    }
}
impl<V: Vertex, I: Index> MeshAttacher for IndexedMesh<V, I> {
    fn attach_with_step<'a, T: Mesh>(&'a self, other: &'a T, step: u32) -> MultiMesh<'a> {
        let mut attachments = self.attachments();
        attachments.extend(other.attachments().into_iter().map(|mut a| {
            a.step = step;
            a
        }));
        MultiMesh {
            ibo: Some((&self.ibo, I::GL_TYPE)),
            attachments,
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
