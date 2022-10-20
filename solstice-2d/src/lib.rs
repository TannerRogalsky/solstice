mod d2;
mod d3;
mod shared;

pub use d2::*;
pub use d3::*;
pub use shared::*;
pub use solstice;

use solstice::{
    image::Image,
    mesh::{MappedIndexedMesh, MappedVertexMesh},
    texture::Texture,
    Context,
};

pub struct GraphicsLock<'a, 'b> {
    ctx: &'a mut Context,
    gfx: &'a mut Graphics,
    dl: DrawList<'b>,
}

impl GraphicsLock<'_, '_> {
    pub fn ctx_mut(&mut self) -> &mut Context {
        self.ctx
    }

    pub fn gfx(&self) -> &Graphics {
        self.gfx
    }
}

impl<'b> std::ops::Deref for GraphicsLock<'_, 'b> {
    type Target = DrawList<'b>;

    fn deref(&self) -> &Self::Target {
        &self.dl
    }
}

impl std::ops::DerefMut for GraphicsLock<'_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.dl
    }
}

impl std::ops::Drop for GraphicsLock<'_, '_> {
    fn drop(&mut self) {
        self.gfx.process(self.ctx, &mut self.dl)
    }
}

struct GeometryBuffers {
    mesh3d: MappedIndexedMesh<Vertex3D, u32>,
    mesh3d_unindexed: MappedVertexMesh<Vertex3D>,
    mesh2d: MappedIndexedMesh<Vertex2D, u32>,
    mesh2d_unindexed: MappedVertexMesh<Vertex2D>,
}

pub struct Graphics {
    meshes: GeometryBuffers,
    line_workspace: LineWorkspace,
    default_shader: Shader,
    default_texture: Image,
    text_workspace: text::Text,
    text_shader: Shader,
    viewport: solstice::viewport::Viewport<i32>,
    scissor: Option<solstice::viewport::Viewport<i32>>,
    default_projection_bounds: Option<Rectangle>,
}

impl Graphics {
    pub fn new(ctx: &mut Context, width: f32, height: f32) -> Result<Self, GraphicsError> {
        let mesh2d = MappedIndexedMesh::new(ctx, 10000, 10000)?;
        let mesh2d_unindexed = MappedVertexMesh::new(ctx, 10000)?;
        let mesh3d = MappedIndexedMesh::new(ctx, 10000, 10000)?;
        let mesh3d_unindexed = MappedVertexMesh::new(ctx, 10000)?;
        let line_workspace = LineWorkspace::new(ctx)?;
        let default_shader = Shader::new(ctx)?;
        let default_texture = create_default_texture(ctx)?;

        let text_workspace = text::Text::new(ctx)?;
        let text_shader = Shader::with((text::DEFAULT_VERT, text::DEFAULT_FRAG), ctx)?;

        Ok(Self {
            meshes: GeometryBuffers {
                mesh3d,
                mesh3d_unindexed,
                mesh2d,
                mesh2d_unindexed,
            },
            line_workspace,
            default_shader,
            default_texture,
            text_workspace,
            text_shader,
            viewport: solstice::viewport::Viewport::new(0, 0, width as _, height as _),
            scissor: None,
            default_projection_bounds: None,
        })
    }

    pub fn lock<'a>(&'a mut self, ctx: &'a mut Context) -> GraphicsLock<'a, '_> {
        GraphicsLock {
            ctx,
            gfx: self,
            dl: Default::default(),
        }
    }

    pub fn add_font(&mut self, font_data: text::FontVec) -> glyph_brush::FontId {
        self.text_workspace.add_font(font_data)
    }

    pub fn set_default_projection_bounds(&mut self, bounds: Option<Rectangle>) {
        self.default_projection_bounds = bounds;
    }

    pub fn set_width_height(&mut self, width: f32, height: f32) {
        self.set_viewport(solstice::viewport::Viewport::new(
            0,
            0,
            width as _,
            height as _,
        ))
    }

    pub fn set_viewport(&mut self, viewport: solstice::viewport::Viewport<i32>) {
        self.viewport = viewport;
    }

    pub fn viewport(&self) -> &solstice::viewport::Viewport<i32> {
        &self.viewport
    }

    pub fn set_scissor(&mut self, scissor: Option<solstice::viewport::Viewport<i32>>) {
        self.scissor = scissor;
    }

    pub fn process(&mut self, ctx: &mut Context, draw_list: &DrawList) {
        fn canvas_bounds(t: &Canvas) -> solstice::viewport::Viewport<i32> {
            let (w, h) = t.dimensions();
            solstice::viewport::Viewport::new(0, 0, w as _, h as _)
        }

        for command in draw_list.commands.iter() {
            match command {
                Command::Draw(draw_state) => {
                    let DrawState {
                        data: geometry,
                        transform,
                        camera,
                        projection_mode,
                        color,
                        texture,
                        target,
                        shader,
                    } = draw_state;

                    let (default_projection_bounds, scissor_state) = if target.is_some() {
                        (None, None)
                    } else {
                        (self.default_projection_bounds, self.scissor)
                    };

                    match geometry {
                        GeometryVariants::D2(geometry) => {
                            let mut shader = shader.clone();
                            let shader = shader.as_mut().unwrap_or(&mut self.default_shader);
                            let viewport = target.as_ref().map_or(self.viewport, canvas_bounds);
                            shader.set_viewport(
                                *projection_mode,
                                default_projection_bounds,
                                viewport,
                                target.is_some(),
                            );
                            shader.set_view(camera);
                            shader.set_model(*transform);
                            shader.set_color(*color);
                            match texture.as_ref() {
                                None => shader.bind_texture(&self.default_texture),
                                Some(texture) => shader.bind_texture(texture),
                            }
                            shader.activate(ctx);
                            ctx.set_viewport(
                                viewport.x() as _,
                                viewport.y() as _,
                                viewport.width() as _,
                                viewport.height() as _,
                            );

                            let settings = solstice::PipelineSettings {
                                depth_state: None,
                                scissor_state,
                                framebuffer: target.as_ref().map(|c| &c.inner),
                                ..solstice::PipelineSettings::default()
                            };
                            geometry.draw(&mut self.meshes, ctx, shader, settings);
                        }
                        GeometryVariants::D3(geometry) => {
                            let mut shader = shader.clone();
                            let shader = shader.as_mut().unwrap_or(&mut self.default_shader);
                            let viewport = target.as_ref().map_or(self.viewport, canvas_bounds);
                            shader.set_viewport(
                                *projection_mode,
                                default_projection_bounds,
                                viewport,
                                target.is_some(),
                            );
                            shader.set_view(camera);
                            shader.set_model(*transform);
                            shader.set_color(draw_state.color);
                            match texture.as_ref() {
                                None => shader.bind_texture(&self.default_texture),
                                Some(texture) => shader.bind_texture(texture),
                            }
                            shader.activate(ctx);

                            ctx.set_viewport(
                                viewport.x(),
                                viewport.y(),
                                viewport.width(),
                                viewport.height(),
                            );

                            let settings = solstice::PipelineSettings {
                                scissor_state,
                                framebuffer: target.as_ref().map(|c| &c.inner),
                                ..solstice::PipelineSettings::default()
                            };
                            geometry.draw(&mut self.meshes, ctx, shader, settings);
                        }
                    };
                }
                Command::Line(draw_state) => {
                    let DrawState {
                        data:
                            LineState {
                                geometry,
                                is_loop,
                                depth_buffer,
                            },
                        transform,
                        camera,
                        projection_mode,
                        color,
                        texture,
                        target,
                        shader,
                    } = draw_state;
                    self.line_workspace.add_points(geometry);
                    if let Some(first) = geometry.first() {
                        if *is_loop {
                            self.line_workspace.add_points(&[*first]);
                        }
                    }

                    let (default_projection_bounds, scissor_state) = if target.is_some() {
                        (None, None)
                    } else {
                        (self.default_projection_bounds, self.scissor)
                    };

                    let shader = shader.clone();
                    let mut shader = shader.unwrap_or_else(|| self.line_workspace.shader().clone());
                    let viewport = target.as_ref().map_or(self.viewport, canvas_bounds);
                    shader.set_viewport(
                        *projection_mode,
                        default_projection_bounds,
                        viewport,
                        target.is_some(),
                    );
                    shader.set_view(camera);
                    shader.set_model(*transform);
                    match texture.as_ref() {
                        None => shader.bind_texture(&self.default_texture),
                        Some(texture) => shader.bind_texture(texture),
                    }
                    shader.set_color(*color);
                    shader.activate(ctx);

                    let geometry = self.line_workspace.geometry(ctx);

                    let depth_state = if *depth_buffer {
                        Some(solstice::DepthState::default())
                    } else {
                        None
                    };

                    ctx.set_viewport(
                        viewport.x(),
                        viewport.y(),
                        viewport.width(),
                        viewport.height(),
                    );
                    solstice::Renderer::draw(
                        ctx,
                        &shader,
                        &geometry,
                        solstice::PipelineSettings {
                            depth_state,
                            framebuffer: target.as_ref().map(|c| &c.inner),
                            scissor_state,
                            ..solstice::PipelineSettings::default()
                        },
                    );
                }
                Command::Print(state) => {
                    let DrawState {
                        data:
                            PrintState {
                                text,
                                font_id,
                                scale,
                                bounds,
                                layout,
                            },
                        transform,
                        camera,
                        projection_mode,
                        color,
                        texture: _,
                        target,
                        shader,
                    } = state;
                    self.text_workspace.set_text(
                        glyph_brush::Text {
                            text,
                            scale: glyph_brush::ab_glyph::PxScale::from(*scale),
                            font_id: *font_id,
                            extra: glyph_brush::Extra {
                                color: (*color).into(),
                                z: 0.0,
                            },
                        },
                        *bounds,
                        layout.into(),
                        ctx,
                    );

                    let (default_projection_bounds, scissor_state) = if target.is_some() {
                        (None, None)
                    } else {
                        (self.default_projection_bounds, self.scissor)
                    };

                    let mut shader = shader.clone();
                    let shader = shader.as_mut().unwrap_or(&mut self.text_shader);
                    shader.bind_texture(self.text_workspace.texture());
                    let viewport = target.as_ref().map_or(self.viewport, canvas_bounds);
                    shader.set_viewport(
                        *projection_mode,
                        default_projection_bounds,
                        viewport,
                        target.is_some(),
                    );
                    shader.set_view(camera);
                    shader.set_model(*transform);
                    shader.set_color(Color::new(1., 1., 1., 1.));
                    shader.activate(ctx);

                    let geometry = self.text_workspace.geometry(ctx);

                    ctx.set_viewport(
                        viewport.x(),
                        viewport.y(),
                        viewport.width(),
                        viewport.height(),
                    );
                    solstice::Renderer::draw(
                        ctx,
                        shader,
                        &geometry,
                        solstice::PipelineSettings {
                            depth_state: None,
                            scissor_state,
                            framebuffer: target.as_ref().map(|c| &c.inner),
                            ..solstice::PipelineSettings::default()
                        },
                    );
                }
                Command::Clear(color, target) => {
                    solstice::Renderer::clear(
                        ctx,
                        solstice::ClearSettings {
                            color: Some((*color).into()),
                            target: target.as_ref().map(|c| &c.inner),
                            ..solstice::ClearSettings::default()
                        },
                    );
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum MaybeOwned<'a, T> {
    Borrowed(&'a [T]),
    Owned(Vec<T>),
}

impl<'a, T> std::ops::Deref for MaybeOwned<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        match self {
            MaybeOwned::Borrowed(v) => v,
            MaybeOwned::Owned(v) => v,
        }
    }
}

impl<'a, T> From<std::borrow::Cow<'a, [T]>> for MaybeOwned<'a, T>
where
    [T]: std::borrow::ToOwned<Owned = Vec<T>>,
{
    fn from(v: std::borrow::Cow<'a, [T]>) -> Self {
        match v {
            std::borrow::Cow::Borrowed(v) => MaybeOwned::Borrowed(v),
            std::borrow::Cow::Owned(v) => MaybeOwned::Owned(v),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Geometry<'a, V, I = u32> {
    vertices: MaybeOwned<'a, V>,
    indices: Option<MaybeOwned<'a, I>>,
}

impl<'a, V, I> Geometry<'a, V, I> {
    pub fn new<IV: Into<std::borrow::Cow<'a, [V]>>, II: Into<std::borrow::Cow<'a, [I]>>>(
        vertices: IV,
        indices: Option<II>,
    ) -> Self
    where
        [V]: std::borrow::ToOwned<Owned = Vec<V>>,
        [I]: std::borrow::ToOwned<Owned = Vec<I>>,
    {
        Self {
            vertices: vertices.into().into(),
            indices: indices.map(Into::into).map(Into::into),
        }
    }
}

pub trait Draw<V: solstice::vertex::Vertex, G> {
    fn draw(&mut self, geometry: G);
    fn draw_with_transform<TX>(&mut self, geometry: G, transform: TX)
    where
        TX: Into<mint::ColumnMatrix4<f32>>;
    fn draw_with_color<C: Into<Color>>(&mut self, geometry: G, color: C);
    fn draw_with_color_and_transform<C, TX>(&mut self, geometry: G, color: C, transform: TX)
    where
        C: Into<Color>,
        TX: Into<mint::ColumnMatrix4<f32>>;
    fn image<T: Texture>(&mut self, geometry: G, texture: T);
    fn image_with_color<T, C>(&mut self, geometry: G, texture: T, color: C)
    where
        T: Texture,
        C: Into<Color>;
    fn image_with_transform<T, TX>(&mut self, geometry: G, texture: T, transform: TX)
    where
        T: Texture,
        TX: Into<mint::ColumnMatrix4<f32>>;
    fn image_with_color_and_transform<T, C, TX>(
        &mut self,
        geometry: G,
        texture: T,
        color: C,
        transform: TX,
    ) where
        T: Texture,
        C: Into<Color>,
        TX: Into<mint::ColumnMatrix4<f32>>;
}
pub trait Stroke<V: solstice::vertex::Vertex, G> {
    fn stroke(&mut self, geometry: G);
    fn stroke_with_transform<TX>(&mut self, geometry: G, transform: TX)
    where
        TX: Into<mint::ColumnMatrix4<f32>>;
    fn stroke_with_color<C: Into<Color>>(&mut self, geometry: G, color: C);
    fn stroke_with_color_and_transform<C, TX>(&mut self, geometry: G, color: C, transform: TX)
    where
        C: Into<Color>,
        TX: Into<mint::ColumnMatrix4<f32>>;
}

#[derive(PartialEq, Clone, Debug)]
struct TextureCache {
    ty: solstice::texture::TextureType,
    key: solstice::TextureKey,
    info: solstice::texture::TextureInfo,
}

impl<T> From<T> for TextureCache
where
    T: solstice::texture::Texture,
{
    fn from(texture: T) -> Self {
        TextureCache {
            ty: texture.get_texture_type(),
            key: texture.get_texture_key(),
            info: texture.get_texture_info(),
        }
    }
}

impl solstice::texture::Texture for &TextureCache {
    fn get_texture_key(&self) -> solstice::TextureKey {
        self.key
    }

    fn get_texture_type(&self) -> solstice::texture::TextureType {
        self.ty
    }

    fn get_texture_info(&self) -> solstice::texture::TextureInfo {
        self.info
    }
}

trait WriteAndDrawBuffer {
    fn draw<S>(
        self,
        meshes: &mut GeometryBuffers,
        ctx: &mut Context,
        shader: &S,
        settings: solstice::PipelineSettings,
    ) where
        S: solstice::shader::Shader;
}

impl WriteAndDrawBuffer for &Geometry<'_, Vertex2D> {
    fn draw<S>(
        self,
        meshes: &mut GeometryBuffers,
        ctx: &mut Context,
        shader: &S,
        settings: solstice::PipelineSettings,
    ) where
        S: solstice::shader::Shader,
    {
        match &self.indices {
            None => {
                meshes.mesh2d_unindexed.set_vertices(&self.vertices, 0);
                let mesh = meshes.mesh2d_unindexed.unmap(ctx);
                let geometry = solstice::Geometry {
                    mesh,
                    draw_range: 0..self.vertices.len(),
                    draw_mode: solstice::DrawMode::Triangles,
                    instance_count: 1,
                };
                solstice::Renderer::draw(ctx, shader, &geometry, settings);
            }
            Some(indices) => {
                meshes.mesh2d.set_vertices(&self.vertices, 0);
                meshes.mesh2d.set_indices(&indices, 0);
                let mesh = meshes.mesh2d.unmap(ctx);
                let geometry = solstice::Geometry {
                    mesh,
                    draw_range: 0..indices.len(),
                    draw_mode: solstice::DrawMode::Triangles,
                    instance_count: 1,
                };
                solstice::Renderer::draw(ctx, shader, &geometry, settings);
            }
        }
    }
}

impl WriteAndDrawBuffer for &Geometry<'_, Vertex3D> {
    fn draw<S>(
        self,
        meshes: &mut GeometryBuffers,
        ctx: &mut Context,
        shader: &S,
        settings: solstice::PipelineSettings,
    ) where
        S: solstice::shader::Shader,
    {
        match &self.indices {
            None => {
                meshes.mesh3d_unindexed.set_vertices(&self.vertices, 0);
                let mesh = meshes.mesh3d_unindexed.unmap(ctx);
                let geometry = solstice::Geometry {
                    mesh,
                    draw_range: 0..self.vertices.len(),
                    draw_mode: solstice::DrawMode::Triangles,
                    instance_count: 1,
                };
                solstice::Renderer::draw(ctx, shader, &geometry, settings);
            }
            Some(indices) => {
                meshes.mesh3d.set_vertices(&self.vertices, 0);
                meshes.mesh3d.set_indices(&indices, 0);
                let mesh = meshes.mesh3d.unmap(ctx);
                let geometry = solstice::Geometry {
                    mesh,
                    draw_range: 0..indices.len(),
                    draw_mode: solstice::DrawMode::Triangles,
                    instance_count: 1,
                };
                solstice::Renderer::draw(ctx, shader, &geometry, settings);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum MeshVariant<'a, V>
where
    V: solstice::vertex::Vertex,
{
    Data(Geometry<'a, V>),
    VertexMesh(solstice::Geometry<&'a solstice::mesh::VertexMesh<V>>),
    IndexedMesh(solstice::Geometry<&'a solstice::mesh::IndexedMesh<V, u32>>),
    IndexedMeshU16(solstice::Geometry<&'a solstice::mesh::IndexedMesh<V, u16>>),
    MultiMesh(solstice::Geometry<&'a solstice::mesh::MultiMesh<'a>>),
}

impl<'a, V> MeshVariant<'a, V>
where
    V: solstice::vertex::Vertex,
{
    fn draw<S>(
        &'a self,
        meshes: &mut GeometryBuffers,
        ctx: &mut Context,
        shader: &S,
        settings: solstice::PipelineSettings,
    ) where
        S: solstice::shader::Shader,
        &'a Geometry<'a, V>: WriteAndDrawBuffer,
    {
        use solstice::Renderer;
        match self {
            MeshVariant::Data(data) => data.draw(meshes, ctx, shader, settings),
            MeshVariant::VertexMesh(geometry) => ctx.draw(shader, geometry, settings),
            MeshVariant::IndexedMesh(geometry) => ctx.draw(shader, geometry, settings),
            MeshVariant::IndexedMeshU16(geometry) => ctx.draw(shader, geometry, settings),
            MeshVariant::MultiMesh(geometry) => ctx.draw(shader, &geometry, settings),
        }
    }
}

pub trait GeometryKind<'a, V>: Sized + std::cmp::PartialEq + Into<MeshVariant<'a, V>>
where
    V: solstice::vertex::Vertex,
{
}
impl<'a, V, T> GeometryKind<'a, V> for T
where
    T: Sized + std::cmp::PartialEq + Into<MeshVariant<'a, V>>,
    V: solstice::vertex::Vertex,
{
}

#[derive(Clone, Debug)]
pub enum GeometryVariants<'a> {
    D2(MeshVariant<'a, Vertex2D>),
    D3(MeshVariant<'a, Vertex3D>),
}

impl<'a, T> From<T> for MeshVariant<'a, Vertex3D>
where
    T: Into<Geometry<'a, Vertex3D>>,
{
    fn from(data: T) -> Self {
        Self::Data(data.into())
    }
}

impl<'a, T> From<T> for MeshVariant<'a, Vertex2D>
where
    T: Into<Geometry<'a, Vertex2D>>,
{
    fn from(data: T) -> Self {
        Self::Data(data.into())
    }
}

impl<'a, V> From<solstice::Geometry<&'a solstice::mesh::VertexMesh<V>>> for MeshVariant<'a, V>
where
    V: solstice::vertex::Vertex,
{
    fn from(v: solstice::Geometry<&'a solstice::mesh::VertexMesh<V>>) -> Self {
        Self::VertexMesh(v)
    }
}

impl<'a, V> From<solstice::Geometry<&'a solstice::mesh::IndexedMesh<V, u32>>> for MeshVariant<'a, V>
where
    V: solstice::vertex::Vertex,
{
    fn from(v: solstice::Geometry<&'a solstice::mesh::IndexedMesh<V, u32>>) -> Self {
        Self::IndexedMesh(v)
    }
}

impl<'a, V> From<solstice::Geometry<&'a solstice::mesh::IndexedMesh<V, u16>>> for MeshVariant<'a, V>
where
    V: solstice::vertex::Vertex,
{
    fn from(v: solstice::Geometry<&'a solstice::mesh::IndexedMesh<V, u16>>) -> Self {
        Self::IndexedMeshU16(v)
    }
}

impl<'a, V> From<solstice::Geometry<&'a solstice::mesh::MultiMesh<'a>>> for MeshVariant<'a, V>
where
    V: solstice::vertex::Vertex,
{
    fn from(v: solstice::Geometry<&'a solstice::mesh::MultiMesh<'a>>) -> Self {
        Self::MultiMesh(v)
    }
}

#[derive(Clone, Debug)]
pub struct DrawState<T> {
    data: T,
    transform: mint::ColumnMatrix4<f32>,
    camera: Transform3D,
    projection_mode: Projection,
    color: Color,
    texture: Option<TextureCache>,
    target: Option<Canvas>,
    shader: Option<Shader>,
}

#[derive(Clone, Debug)]
pub struct LineState<'a> {
    geometry: std::borrow::Cow<'a, [LineVertex]>,
    is_loop: bool,
    depth_buffer: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

impl From<&HorizontalAlign> for glyph_brush::HorizontalAlign {
    fn from(align: &HorizontalAlign) -> Self {
        match align {
            HorizontalAlign::Left => glyph_brush::HorizontalAlign::Left,
            HorizontalAlign::Center => glyph_brush::HorizontalAlign::Center,
            HorizontalAlign::Right => glyph_brush::HorizontalAlign::Right,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
}

impl From<&VerticalAlign> for glyph_brush::VerticalAlign {
    fn from(align: &VerticalAlign) -> Self {
        match align {
            VerticalAlign::Top => glyph_brush::VerticalAlign::Top,
            VerticalAlign::Center => glyph_brush::VerticalAlign::Center,
            VerticalAlign::Bottom => glyph_brush::VerticalAlign::Bottom,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrintLayout {
    SingleLine {
        h_align: HorizontalAlign,
        v_align: VerticalAlign,
    },
    Wrap {
        h_align: HorizontalAlign,
        v_align: VerticalAlign,
    },
}

impl std::default::Default for PrintLayout {
    fn default() -> Self {
        PrintLayout::Wrap {
            h_align: HorizontalAlign::Left,
            v_align: VerticalAlign::Top,
        }
    }
}

impl From<&PrintLayout> for glyph_brush::Layout<glyph_brush::BuiltInLineBreaker> {
    fn from(layout: &PrintLayout) -> Self {
        match layout {
            PrintLayout::SingleLine { h_align, v_align } => glyph_brush::Layout::SingleLine {
                line_breaker: glyph_brush::BuiltInLineBreaker::default(),
                h_align: h_align.into(),
                v_align: v_align.into(),
            },
            PrintLayout::Wrap { h_align, v_align } => glyph_brush::Layout::Wrap {
                line_breaker: glyph_brush::BuiltInLineBreaker::default(),
                h_align: h_align.into(),
                v_align: v_align.into(),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct PrintState<'a> {
    text: std::borrow::Cow<'a, str>,
    font_id: glyph_brush::FontId,
    scale: f32,
    bounds: d2::Rectangle,
    layout: PrintLayout,
}

#[derive(Clone, Debug)]
pub enum Command<'a> {
    Draw(DrawState<GeometryVariants<'a>>),
    Print(DrawState<PrintState<'a>>),
    Line(DrawState<LineState<'a>>),
    Clear(Color, Option<Canvas>),
}

#[derive(Clone, Debug)]
pub struct DrawList<'a> {
    commands: Vec<Command<'a>>,
    color: Color,
    transform: mint::ColumnMatrix4<f32>,
    line_width: f32,
    camera: Transform3D,
    projection_mode: Option<Projection>,
    target: Option<Canvas>,
    shader: Option<Shader>,
}

impl Default for DrawList<'_> {
    fn default() -> Self {
        Self {
            commands: vec![],
            color: Default::default(),
            transform: Transform3D::default().into(),
            line_width: 1.0,
            camera: Default::default(),
            projection_mode: None,
            target: None,
            shader: None,
        }
    }
}

impl<'a> DrawList<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_from_state<'b>(other: &DrawList) -> DrawList<'b> {
        DrawList {
            commands: vec![],
            color: other.color,
            transform: other.transform,
            line_width: 1.0,
            camera: other.camera,
            projection_mode: other.projection_mode,
            target: other.target.clone(),
            shader: other.shader.clone(),
        }
    }

    pub fn append(&mut self, other: &mut Self) {
        self.commands.append(&mut other.commands);
    }

    pub fn clear<C: Into<Color>>(&mut self, color: C) {
        let command = Command::Clear(color.into(), self.target.clone());
        self.commands.push(command)
    }

    pub fn print<T>(&mut self, text: T, font_id: glyph_brush::FontId, scale: f32, bounds: Rectangle)
    where
        T: Into<std::borrow::Cow<'a, str>>,
    {
        let command = Command::Print(DrawState {
            data: PrintState {
                text: text.into(),
                font_id,
                scale,
                bounds,
                layout: Default::default(),
            },
            transform: self.transform.into(),
            camera: self.camera,
            projection_mode: self
                .projection_mode
                .unwrap_or(Projection::Orthographic(None)),
            color: self.color,
            texture: None,
            target: self.target.clone(),
            shader: self.shader.clone(),
        });
        self.commands.push(command);
    }

    pub fn print_with_layout<T>(
        &mut self,
        text: T,
        font_id: glyph_brush::FontId,
        scale: f32,
        bounds: Rectangle,
        layout: PrintLayout,
    ) where
        T: Into<std::borrow::Cow<'a, str>>,
    {
        let command = Command::Print(DrawState {
            data: PrintState {
                text: text.into(),
                font_id,
                scale,
                bounds,
                layout,
            },
            transform: self.transform.into(),
            camera: self.camera,
            projection_mode: self
                .projection_mode
                .unwrap_or(Projection::Orthographic(None)),
            color: self.color,
            texture: None,
            target: self.target.clone(),
            shader: self.shader.clone(),
        });
        self.commands.push(command);
    }

    pub fn line_2d<G>(&mut self, points: G)
    where
        G: Into<std::borrow::Cow<'a, [LineVertex]>>,
    {
        let command = Command::Line(DrawState {
            data: LineState {
                geometry: points.into(),
                is_loop: false,
                depth_buffer: false,
            },
            transform: self.transform.into(),
            camera: self.camera,
            projection_mode: self
                .projection_mode
                .unwrap_or(Projection::Orthographic(None)),
            color: self.color,
            texture: None,
            target: self.target.clone(),
            shader: self.shader.clone(),
        });
        self.commands.push(command)
    }

    pub fn line_3d<G>(&mut self, points: G)
    where
        G: Into<std::borrow::Cow<'a, [LineVertex]>>,
    {
        let command = Command::Line(DrawState {
            data: LineState {
                geometry: points.into(),
                is_loop: false,
                depth_buffer: true,
            },
            transform: self.transform.into(),
            camera: self.camera,
            projection_mode: self
                .projection_mode
                .unwrap_or(Projection::Perspective(None)),
            color: self.color,
            texture: None,
            target: self.target.clone(),
            shader: self.shader.clone(),
        });
        self.commands.push(command)
    }

    pub fn set_color<C: Into<Color>>(&mut self, color: C) {
        self.color = color.into();
    }

    pub fn set_transform<T: Into<mint::ColumnMatrix4<f32>>>(&mut self, transform: T) {
        self.transform = transform.into();
    }

    pub fn set_line_width(&mut self, line_width: f32) {
        self.line_width = line_width;
    }

    pub fn set_camera<T: Into<Transform3D>>(&mut self, camera: T) {
        self.camera = camera.into();
    }

    pub fn set_projection_mode(&mut self, projection_mode: Option<Projection>) {
        self.projection_mode = projection_mode;
    }

    pub fn set_canvas(&mut self, target: Option<Canvas>) {
        self.target = target;
    }

    pub fn set_shader(&mut self, shader: Option<Shader>) {
        self.shader = shader;
    }
}

impl<'a> DrawList<'a> {
    fn push_draw(
        &mut self,
        data: GeometryVariants<'a>,
        color: Color,
        transform: mint::ColumnMatrix4<f32>,
        texture: Option<TextureCache>,
    ) {
        let projection_mode = self.projection_mode.unwrap_or_else(|| match &data {
            GeometryVariants::D2(_) => Projection::Orthographic(None),
            GeometryVariants::D3(_) => Projection::Perspective(None),
        });
        self.commands.push(Command::Draw(DrawState {
            data,
            transform,
            camera: self.camera,
            projection_mode,
            color,
            texture,
            target: self.target.clone(),
            shader: self.shader.clone(),
        }))
    }
}

type ImageResult = Result<solstice::image::Image, solstice::GraphicsError>;

pub fn create_default_texture(gl: &mut solstice::Context) -> ImageResult {
    use solstice::image::*;
    use solstice::texture::*;
    Image::with_data(
        gl,
        TextureType::Tex2D,
        solstice::PixelFormat::RGBA8,
        1,
        1,
        &[255, 255, 255, 255],
        Settings {
            mipmaps: false,
            filter: FilterMode::Nearest,
            wrap: WrapMode::Clamp,
            ..Settings::default()
        },
    )
}
