use solstice::{mesh::MappedIndexedMesh, texture::Texture, Context};

mod canvas;
mod color;
mod shader;
mod shapes;
mod text;
mod transforms;
mod vertex;

pub use canvas::Canvas;
pub use color::*;
pub use glyph_brush::{ab_glyph::FontVec, FontId};
pub use shader::Shader2D;
pub use shapes::*;
pub use transforms::*;
pub use vertex::{Point, Vertex2D};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DrawMode {
    Fill,
    Stroke,
}

#[derive(Debug)]
pub enum Graphics2DError {
    ShaderError(shader::Shader2DError),
    GraphicsError(solstice::GraphicsError),
}

impl std::fmt::Display for Graphics2DError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Graphics2DError {}

impl From<solstice::GraphicsError> for Graphics2DError {
    fn from(err: solstice::GraphicsError) -> Self {
        Graphics2DError::GraphicsError(err)
    }
}

impl From<shader::Shader2DError> for Graphics2DError {
    fn from(err: shader::Shader2DError) -> Self {
        Graphics2DError::ShaderError(err)
    }
}

#[must_use]
pub struct Graphics2DLock<'a, 'b> {
    ctx: &'a mut Context,
    inner: &'b mut Graphics2D,
    draw_list: DrawList,
}

impl std::ops::Deref for Graphics2DLock<'_, '_> {
    type Target = DrawList;

    fn deref(&self) -> &Self::Target {
        &self.draw_list
    }
}

impl std::ops::DerefMut for Graphics2DLock<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.draw_list
    }
}

impl Drop for Graphics2DLock<'_, '_> {
    fn drop(&mut self) {
        self.inner.process(&mut self.draw_list, self.ctx);
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
struct TextureCache {
    ty: solstice::texture::TextureType,
    key: solstice::TextureKey,
}

#[derive(Debug, Clone)]
pub struct DrawCommand {
    shader: Option<Shader2D>,
    target: Option<Canvas>,
    texture: Option<TextureCache>,
    geometry: Box<dyn AnyGeometry<'static, Vertex2D, u32>>,
    draw_mode: DrawMode,
    color: Color,
    transform: Transform,
}

#[derive(Debug, Clone)]
pub struct PrintCommand {
    font: glyph_brush::FontId,
    text: String,
    x: f32,
    y: f32,
    scale: f32,
    color: Color,
}

#[derive(Debug, Clone)]
pub enum Command {
    Draw(DrawCommand),
    Print(PrintCommand),
    Clear(Color),
}

#[derive(Debug, Clone, Default)]
pub struct DrawList {
    commands: Vec<Command>,
    color: Color,
    transform: Transform,
    shader: Option<Shader2D>,
    target: Option<Canvas>,
}

pub trait ConcreteGeometry: Geometry + Clone + 'static {}
impl<T> ConcreteGeometry for T where T: Geometry + Clone + 'static {}

impl DrawList {
    pub fn set_color<C: Into<Color>>(&mut self, color: C) {
        self.color = color.into();
    }

    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    pub fn set_shader(&mut self, shader: Option<Shader2D>) {
        self.shader = shader;
    }

    pub fn set_canvas(&mut self, target: Option<Canvas>) {
        self.target = target;
    }

    pub fn draw<G: ConcreteGeometry>(&mut self, draw_mode: DrawMode, geometry: G) {
        self.draw_with_transform_and_color(draw_mode, geometry, self.transform, self.color)
    }

    pub fn draw_with_transform<G>(&mut self, draw_mode: DrawMode, geometry: G, transform: Transform)
    where
        G: ConcreteGeometry,
    {
        self.draw_with_transform_and_color(draw_mode, geometry, transform, self.color)
    }

    pub fn draw_with_color<G, C>(&mut self, draw_mode: DrawMode, geometry: G, color: C)
    where
        G: ConcreteGeometry,
        C: Into<Color>,
    {
        self.draw_with_transform_and_color(draw_mode, geometry, self.transform, color)
    }

    pub fn draw_with_transform_and_color<G, C>(
        &mut self,
        draw_mode: DrawMode,
        geometry: G,
        transform: Transform,
        color: C,
    ) where
        G: ConcreteGeometry,
        C: Into<Color>,
    {
        self.commands.push(Command::Draw(DrawCommand {
            shader: None,
            target: None,
            texture: None,
            geometry: Box::new(geometry),
            draw_mode,
            color: color.into(),
            transform,
        }))
    }

    pub fn image<G, T>(&mut self, geometry: G, texture: T)
    where
        G: ConcreteGeometry,
        T: Texture + Copy,
    {
        self.image_with_transform_and_color(geometry, texture, self.transform, self.color)
    }

    pub fn image_with_transform<G, T>(&mut self, geometry: G, texture: T, transform: Transform)
    where
        G: ConcreteGeometry,
        T: Texture + Copy,
    {
        self.image_with_transform_and_color(geometry, texture, transform, self.color)
    }

    pub fn image_with_color<G, T, C>(&mut self, geometry: G, texture: T, color: C)
    where
        G: ConcreteGeometry,
        T: Texture + Copy,
        C: Into<Color>,
    {
        self.image_with_transform_and_color(geometry, texture, self.transform, color)
    }

    pub fn image_with_transform_and_color<G, T, C>(
        &mut self,
        geometry: G,
        texture: T,
        transform: Transform,
        color: C,
    ) where
        G: ConcreteGeometry,
        T: Texture + Copy,
        C: Into<Color>,
    {
        self.commands.push(Command::Draw(DrawCommand {
            shader: None,
            target: None,
            texture: Some(TextureCache {
                ty: texture.get_texture_type(),
                key: texture.get_texture_key(),
            }),
            geometry: Box::new(geometry),
            draw_mode: DrawMode::Fill,
            color: color.into(),
            transform,
        }))
    }

    pub fn print<S>(&mut self, font: glyph_brush::FontId, text: S, x: f32, y: f32, scale: f32)
    where
        S: AsRef<str>,
    {
        self.commands.push(Command::Print(PrintCommand {
            font,
            text: text.as_ref().to_string(),
            x,
            y,
            scale,
            color: self.color,
        }))
    }

    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.commands.push(Command::Draw(DrawCommand {
            shader: None,
            target: None,
            texture: None,
            geometry: Box::new([Point::new(x1, y1), Point::new(x2, y2)]),
            draw_mode: DrawMode::Stroke,
            color: self.color,
            transform: self.transform,
        }))
    }
    pub fn lines<G: SimpleConvexGeometry + Clone + 'static>(&mut self, points: G) {
        self.commands.push(Command::Draw(DrawCommand {
            shader: None,
            target: None,
            texture: None,
            geometry: Box::new(points),
            draw_mode: DrawMode::Stroke,
            color: self.color,
            transform: self.transform,
        }))
    }

    pub fn clear<C: Into<Color>>(&mut self, color: C) {
        self.commands.push(Command::Clear(color.into()))
    }
}

#[derive(Default, Debug)]
struct DrawState {
    shader: Option<Shader2D>,
    target: Option<Canvas>,
}

pub struct Graphics2D {
    mesh: MappedIndexedMesh<Vertex2D, u32>,
    default_shader: Shader2D,
    default_texture: solstice::image::Image,
    text_workspace: text::Text,
    text_shader: Shader2D,
    width: f32,
    height: f32,
}

impl Graphics2D {
    pub fn new(ctx: &mut Context, width: f32, height: f32) -> Result<Self, Graphics2DError> {
        let mesh = MappedIndexedMesh::new(ctx, 10000, 10000)?;
        let default_shader = Shader2D::new(ctx)?;
        let default_texture = super::create_default_texture(ctx)?;
        let text_workspace = text::Text::new(ctx)?;
        let text_shader = super::Shader2D::with((text::DEFAULT_VERT, text::DEFAULT_FRAG), ctx)?;
        Ok(Self {
            mesh,
            default_shader,
            default_texture,
            text_workspace,
            text_shader,
            width,
            height,
        })
    }

    pub fn start<'a>(&'a mut self, ctx: &'a mut Context) -> Graphics2DLock<'a, '_> {
        Graphics2DLock {
            ctx,
            inner: self,
            draw_list: Default::default(),
        }
    }

    fn flush(&mut self, ctx: &mut Context, index_offset: usize, draw_state: &mut DrawState) {
        let mesh = self.mesh.unmap(ctx);

        let geometry = solstice::Geometry {
            mesh,
            draw_range: 0..index_offset,
            draw_mode: solstice::DrawMode::Triangles,
            instance_count: 1,
        };

        let shader = match draw_state.shader.as_mut() {
            None => &mut self.default_shader,
            Some(shader) => shader,
        };

        let viewport = ctx.viewport();
        match draw_state.target.as_ref() {
            None => {
                shader.set_width_height(self.width, self.height, false);
            }
            Some(canvas) => {
                let (width, height) = canvas.dimensions();
                // TODO: this sort of thing might be better handled with a whole state stack push/pop
                ctx.set_viewport(0, 0, width as _, height as _);
                shader.set_width_height(width, height, true);
            }
        }

        shader.activate(ctx);
        solstice::Renderer::draw(
            ctx,
            shader,
            &geometry,
            solstice::PipelineSettings {
                depth_state: None,
                framebuffer: draw_state.target.as_ref().map(|c| &c.inner),
                ..solstice::PipelineSettings::default()
            },
        );

        // rollback the viewport change
        if draw_state.target.is_some() {
            ctx.set_viewport(
                viewport.x(),
                viewport.y(),
                viewport.width(),
                viewport.height(),
            );
        }
    }

    pub fn process(&mut self, draw_list: &mut DrawList, ctx: &mut Context) {
        let mut draw_state = DrawState::default();
        let mut vertex_offset = 0;
        let mut index_offset = 0;

        for command in draw_list.commands.drain(..) {
            match command {
                Command::Draw(mut command) => {
                    let geometry = &command.geometry;

                    // TODO: Can we do this on the GPU with a separate buffer and/or instancing?
                    let transform = command.transform;
                    let color = command.color.into();
                    let transform_vertex = move |v: Vertex2D| {
                        let (x, y) = transform.transform_point(v.position[0], v.position[1]);
                        Vertex2D {
                            position: [x, y],
                            color,
                            ..v
                        }
                    };

                    let (vertices, mut indices) = match command.draw_mode {
                        DrawMode::Stroke => {
                            let vertices = geometry.vertices().map(transform_vertex);
                            stroke_polygon(vertices, command.color.into())
                        }
                        DrawMode::Fill => (
                            geometry.vertices().map(transform_vertex).collect(),
                            geometry.indices().collect(),
                        ),
                    };

                    let shader = command.shader.as_mut().unwrap_or(&mut self.default_shader);
                    shader.bind_texture(&self.default_texture);

                    let will_overflow = vertex_offset + vertices.len()
                        > self.mesh.vertex_capacity()
                        || index_offset + indices.len() > self.mesh.index_capacity();
                    if will_overflow {
                        self.flush(ctx, index_offset, &mut draw_state);
                        vertex_offset = 0;
                        index_offset = 0;
                    }

                    for index in indices.iter_mut() {
                        *index += vertex_offset as u32;
                    }

                    self.mesh.set_vertices(&vertices, vertex_offset);
                    self.mesh.set_indices(&indices, index_offset);
                    vertex_offset += vertices.len();
                    index_offset += indices.len();
                }
                Command::Print(PrintCommand {
                    font,
                    text,
                    x,
                    y,
                    scale,
                    color,
                }) => {
                    self.flush(ctx, index_offset, &mut draw_state);
                    let bounds = Rectangle {
                        x,
                        y,
                        width: self.width - x,
                        height: self.height - y,
                    };
                    let text = glyph_brush::Text::new(text.as_str())
                        .with_color(color)
                        .with_font_id(font)
                        .with_scale(scale);
                    self.text_workspace.set_text(text, bounds, ctx);

                    let invert_y = draw_state.target.is_some();
                    let shader = &mut self.text_shader;
                    shader.set_width_height(self.width, self.height, invert_y);
                    shader.bind_texture(self.text_workspace.texture());
                    shader.activate(ctx);
                    let geometry = self.text_workspace.geometry(ctx);
                    solstice::Renderer::draw(
                        ctx,
                        shader,
                        &geometry,
                        solstice::PipelineSettings {
                            depth_state: None,
                            ..solstice::PipelineSettings::default()
                        },
                    );
                }
                Command::Clear(color) => solstice::Renderer::clear(
                    ctx,
                    solstice::ClearSettings {
                        color: Some(color.into()),
                        depth: None,
                        stencil: None,
                        target: None,
                    },
                ),
            }
        }
        self.flush(ctx, index_offset, &mut draw_state);
    }

    pub fn add_font(&mut self, font_data: glyph_brush::ab_glyph::FontVec) -> glyph_brush::FontId {
        self.text_workspace.add_font(font_data)
    }

    pub fn set_width_height(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn dimensions(&self) -> (f32, f32) {
        (self.width, self.height)
    }
}

fn stroke_polygon<P, I>(vertices: I, color: [f32; 4]) -> (Vec<Vertex2D>, Vec<u32>)
where
    P: Into<lyon_tessellation::math::Point>,
    I: IntoIterator<Item = P>,
{
    use lyon_tessellation::*;
    let mut builder = path::Builder::new();
    builder.polygon(&vertices.into_iter().map(Into::into).collect::<Box<[_]>>());
    let path = builder.build();

    struct WithColor([f32; 4]);

    impl StrokeVertexConstructor<Vertex2D> for WithColor {
        fn new_vertex(
            &mut self,
            point: lyon_tessellation::math::Point,
            attributes: StrokeAttributes<'_, '_>,
        ) -> Vertex2D {
            Vertex2D {
                position: [point.x, point.y],
                color: self.0,
                uv: attributes.normal().into(),
            }
        }
    }

    let mut buffers: VertexBuffers<Vertex2D, u32> = VertexBuffers::new();
    {
        let mut tessellator = StrokeTessellator::new();
        tessellator
            .tessellate(
                &path,
                &StrokeOptions::default().with_line_width(5.),
                &mut BuffersBuilder::new(&mut buffers, WithColor(color)),
            )
            .unwrap();
    }

    (buffers.vertices, buffers.indices)
}

pub trait Geometry: std::fmt::Debug {
    type Vertices: Iterator<Item = Vertex2D>;
    type Indices: Iterator<Item = u32>;

    fn vertices(&self) -> Self::Vertices;
    fn indices(&self) -> Self::Indices;
}

pub trait SimpleConvexGeometry: std::fmt::Debug {
    type Vertices: Iterator<Item = Vertex2D>;
    fn vertices(&self) -> Self::Vertices;
    fn vertex_count(&self) -> usize;
}

impl<T: SimpleConvexGeometry> Geometry for T {
    type Vertices = T::Vertices;
    type Indices = std::iter::FlatMap<
        std::ops::Range<u32>,
        arrayvec::ArrayVec<[u32; 3]>,
        fn(u32) -> arrayvec::ArrayVec<[u32; 3]>,
    >;

    fn vertices(&self) -> Self::Vertices {
        T::vertices(self)
    }

    fn indices(&self) -> Self::Indices {
        (1..(self.vertex_count() as u32 - 1))
            .flat_map(|i| arrayvec::ArrayVec::<[u32; 3]>::from([0, i, i + 1]))
    }
}

macro_rules! impl_array_simple_convex_geom {
    ($ty:ty, $count:expr) => {
        impl SimpleConvexGeometry for [$ty; $count] {
            type Vertices = std::iter::Map<std::vec::IntoIter<$ty>, fn($ty) -> Vertex2D>;

            fn vertices(&self) -> Self::Vertices {
                self.to_vec().into_iter().map(Into::into)
            }

            fn vertex_count(&self) -> usize {
                self.len()
            }
        }

        impl SimpleConvexGeometry for &[$ty; $count] {
            type Vertices = std::iter::Map<std::vec::IntoIter<$ty>, fn($ty) -> Vertex2D>;

            fn vertices(&self) -> Self::Vertices {
                self.to_vec().into_iter().map(Into::into)
            }

            fn vertex_count(&self) -> usize {
                self.len()
            }
        }
    };
}

macro_rules! impl_32_array_simple_convex_geom {
    ($ty:ty) => {
        impl_array_simple_convex_geom!($ty, 1);
        impl_array_simple_convex_geom!($ty, 2);
        impl_array_simple_convex_geom!($ty, 3);
        impl_array_simple_convex_geom!($ty, 4);
        impl_array_simple_convex_geom!($ty, 5);
        impl_array_simple_convex_geom!($ty, 6);
        impl_array_simple_convex_geom!($ty, 7);
        impl_array_simple_convex_geom!($ty, 8);
        impl_array_simple_convex_geom!($ty, 9);
        impl_array_simple_convex_geom!($ty, 10);
        impl_array_simple_convex_geom!($ty, 11);
        impl_array_simple_convex_geom!($ty, 12);
        impl_array_simple_convex_geom!($ty, 13);
        impl_array_simple_convex_geom!($ty, 14);
        impl_array_simple_convex_geom!($ty, 15);
        impl_array_simple_convex_geom!($ty, 16);
        impl_array_simple_convex_geom!($ty, 17);
        impl_array_simple_convex_geom!($ty, 18);
        impl_array_simple_convex_geom!($ty, 19);
        impl_array_simple_convex_geom!($ty, 20);
        impl_array_simple_convex_geom!($ty, 21);
        impl_array_simple_convex_geom!($ty, 22);
        impl_array_simple_convex_geom!($ty, 23);
        impl_array_simple_convex_geom!($ty, 24);
        impl_array_simple_convex_geom!($ty, 25);
        impl_array_simple_convex_geom!($ty, 26);
        impl_array_simple_convex_geom!($ty, 27);
        impl_array_simple_convex_geom!($ty, 28);
        impl_array_simple_convex_geom!($ty, 29);
        impl_array_simple_convex_geom!($ty, 30);
        impl_array_simple_convex_geom!($ty, 31);
        impl_array_simple_convex_geom!($ty, 32);
    };
}

impl_32_array_simple_convex_geom!((f32, f32));
impl_32_array_simple_convex_geom!((f64, f64));
impl_32_array_simple_convex_geom!(Point);

impl<'a> SimpleConvexGeometry for &'a [Vertex2D] {
    type Vertices = std::iter::Copied<std::slice::Iter<'a, Vertex2D>>;

    fn vertices(&self) -> Self::Vertices {
        self.into_iter().copied()
    }

    fn vertex_count(&self) -> usize {
        self.len()
    }
}

impl<'a> SimpleConvexGeometry for &'a [(f32, f32)] {
    type Vertices =
        std::iter::Map<std::slice::Iter<'a, (f32, f32)>, fn(&'a (f32, f32)) -> Vertex2D>;

    fn vertices(&self) -> Self::Vertices {
        self.iter().map(|p| (*p).into())
    }

    fn vertex_count(&self) -> usize {
        self.len()
    }
}

impl<'a> SimpleConvexGeometry for &'a [(f64, f64)] {
    type Vertices =
        std::iter::Map<std::slice::Iter<'a, (f64, f64)>, fn(&'a (f64, f64)) -> Vertex2D>;

    fn vertices(&self) -> Self::Vertices {
        self.iter().map(|p| (*p).into())
    }

    fn vertex_count(&self) -> usize {
        self.len()
    }
}

pub trait AnyGeometry<'a, V, I>: dyn_clone::DynClone + std::fmt::Debug
where
    V: solstice::vertex::Vertex,
    I: solstice::mesh::Index,
{
    fn vertices(&self) -> Box<dyn Iterator<Item = V> + 'a>;
    fn indices(&self) -> Box<dyn Iterator<Item = I> + 'a>;
}

impl<'a, V, I, G> AnyGeometry<'a, Vertex2D, u32> for G
where
    V: Iterator<Item = Vertex2D> + 'a,
    I: Iterator<Item = u32> + 'a,
    G: Geometry<Vertices = V, Indices = I> + dyn_clone::DynClone + std::fmt::Debug,
{
    fn vertices(&self) -> Box<dyn Iterator<Item = Vertex2D> + 'a> {
        Box::new(Geometry::vertices(self))
    }

    fn indices(&self) -> Box<dyn Iterator<Item = u32> + 'a> {
        Box::new(Geometry::indices(self))
    }
}
dyn_clone::clone_trait_object!(AnyGeometry<'_, Vertex2D, u32>);
