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

    /// Get a view into the Mesh's vertex data.
    pub fn get_vertices(&self) -> &[V] {
        get_buffer(&self.vbo)
    }

    /// Write new data into a range of the Mesh's vertex data.
    pub fn set_vertices(&mut self, vertices: &[V], offset: usize) {
        set_buffer(&mut self.vbo, vertices, offset);
    }

    /// Sets the range of vertices to be drawn. Passing `None` will draw the entire mesh.
    pub fn set_draw_range(&mut self, draw_range: Option<std::ops::Range<usize>>) {
        self.draw_range = draw_range;
    }

    /// Sets how the vertex data should be tesselated.
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

    /// Draw the mesh.
    pub fn draw(&mut self, gl: &mut Context) {
        self.draw_instanced(gl, 1);
    }

    /// Draw the mesh multiple times using the same vertex data. This might be useful if you're
    /// using per instance data from a uniform or a separate [attached mesh](graphics::mesh::MeshAttacher).
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

        let shader = gl.get_active_shader().expect("No active shader.");
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

    /// Construct an indexed mesh from a non-indexed mesh.
    pub fn with_mesh(
        gl: &mut Context,
        mesh: Mesh<V>,
        index_count: usize,
    ) -> Result<Self, super::GraphicsError> {
        let ibo = Buffer::new(
            gl,
            index_count * std::mem::size_of::<I>(),
            BufferType::Index,
            Usage::Dynamic,
        )?;
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

impl<V> MeshTrait for Mesh<V>
where
    V: Vertex,
{
    fn get_attributes(&mut self) -> AttachedAttributes {
        self.attributes()
    }

    fn secret_draw(
        &mut self,
        gl: &mut Context,
        instance_count: usize,
        attached_attributes: &mut [AttachedAttributes],
    ) {
        self.internal_draw(gl, instance_count, attached_attributes)
    }
}

impl<V, I> MeshTrait for IndexedMesh<V, I>
where
    V: Vertex,
    I: Index,
{
    fn get_attributes(&mut self) -> AttachedAttributes {
        self.attributes()
    }

    fn secret_draw(
        &mut self,
        gl: &mut Context,
        instance_count: usize,
        attached_attributes: &mut [AttachedAttributes],
    ) {
        self.internal_draw(gl, instance_count, attached_attributes)
    }
}

pub trait MeshTrait {
    fn get_attributes(&mut self) -> AttachedAttributes;
    fn secret_draw(
        &mut self,
        gl: &mut Context,
        instance_count: usize,
        attached_attributes: &mut [AttachedAttributes],
    );
}

pub struct AttachedAttributes<'a> {
    buffer: &'a mut Buffer,
    formats: &'a [super::vertex::VertexFormat],
    step: u32,
    stride: usize,
}

pub struct MultiMesh<'a, T> {
    base: &'a mut T,
    attachments: Vec<AttachedAttributes<'a>>,
}

impl<'a, T> MultiMesh<'a, T> {
    pub fn new(base: &'a mut T, attachments: Vec<AttachedAttributes<'a>>) -> Self {
        Self { base, attachments }
    }
}

impl<'a, T> MultiMesh<'a, T>
where
    T: MeshTrait,
{
    pub fn draw_instanced(&mut self, gl: &mut Context, instance_count: usize) {
        self.base
            .secret_draw(gl, instance_count, &mut self.attachments)
    }
}

pub trait MeshAttacher<'a, B>
where
    Self: Sized,
{
    fn attach<T>(self, other: &'a mut T) -> MultiMesh<'a, B>
    where
        T: MeshTrait,
    {
        Self::attach_with_step(self, other, 0)
    }

    fn attach_with_step<T>(self, other: &'a mut T, step: u32) -> MultiMesh<'a, B>
    where
        T: MeshTrait;
}

impl<'a, S> MeshAttacher<'a, S> for &'a mut S {
    fn attach_with_step<T>(self, other: &'a mut T, step: u32) -> MultiMesh<'a, S>
    where
        T: MeshTrait,
    {
        let mut attachments = other.get_attributes();
        attachments.step = step;
        MultiMesh {
            base: self,
            attachments: vec![attachments],
        }
    }
}

impl<'a, B> MeshAttacher<'a, B> for MultiMesh<'a, B> {
    fn attach_with_step<T>(mut self, other: &'a mut T, step: u32) -> MultiMesh<'a, B>
    where
        T: MeshTrait,
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
