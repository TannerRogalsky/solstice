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

pub struct Graphics {
    mesh3d: MappedIndexedMesh<Vertex3D, u32>,
    mesh3d_unindexed: MappedVertexMesh<Vertex3D>,
    mesh2d: MappedIndexedMesh<Vertex2D, u32>,
    mesh2d_unindexed: MappedVertexMesh<Vertex2D>,
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
            mesh3d,
            mesh3d_unindexed,
            mesh2d,
            mesh2d_unindexed,
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

    pub fn add_font(&mut self, font_data: glyph_brush::ab_glyph::FontVec) -> glyph_brush::FontId {
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
                        GeometryVariants::D2(vertices, indices) => {
                            let transform_verts = |v: &Vertex2D| -> Vertex2D {
                                let [x, y] = v.position;
                                let [x, y, _] = transform.transform_point(x, y, 0.);
                                Vertex2D {
                                    position: [x, y],
                                    color: (*color).into(),
                                    uv: v.uv,
                                }
                            };

                            let mut shader = shader.clone();
                            let shader = shader.as_mut().unwrap_or(&mut self.default_shader);
                            let viewport = target.as_ref().map_or(self.viewport, canvas_bounds);
                            shader.set_viewport(
                                *projection_mode,
                                default_projection_bounds,
                                viewport,
                                target.is_some(),
                            );
                            shader.send_uniform(
                                "uView",
                                solstice::shader::RawUniformValue::Mat4(camera.into()),
                            );
                            shader.send_uniform(
                                "uModel",
                                solstice::shader::RawUniformValue::Mat4(
                                    Transform2D::default().into(),
                                ),
                            );
                            shader.set_color(Color::default());
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
                            let vertices = vertices.iter().map(transform_verts).collect::<Vec<_>>();
                            match &indices {
                                None => {
                                    self.mesh2d_unindexed.set_vertices(&vertices, 0);
                                    let mesh = self.mesh2d_unindexed.unmap(ctx);
                                    let geometry = solstice::Geometry {
                                        mesh,
                                        draw_range: 0..vertices.len(),
                                        draw_mode: solstice::DrawMode::Triangles,
                                        instance_count: 1,
                                    };
                                    solstice::Renderer::draw(ctx, shader, &geometry, settings);
                                }
                                Some(indices) => {
                                    self.mesh2d.set_vertices(&vertices, 0);
                                    self.mesh2d.set_indices(&indices, 0);
                                    let mesh = self.mesh2d.unmap(ctx);
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
                        GeometryVariants::D3(vertices, indices) => {
                            let mut shader = shader.clone();
                            let shader = shader.as_mut().unwrap_or(&mut self.default_shader);
                            let viewport = target.as_ref().map_or(self.viewport, canvas_bounds);
                            shader.set_viewport(
                                *projection_mode,
                                default_projection_bounds,
                                viewport,
                                target.is_some(),
                            );
                            shader.send_uniform(
                                "uView",
                                solstice::shader::RawUniformValue::Mat4(camera.into()),
                            );
                            shader.send_uniform(
                                "uModel",
                                solstice::shader::RawUniformValue::Mat4(transform.into()),
                            );
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
                            match &indices {
                                None => {
                                    self.mesh3d_unindexed.set_vertices(&vertices, 0);
                                    let mesh = self.mesh3d_unindexed.unmap(ctx);
                                    let geometry = solstice::Geometry {
                                        mesh,
                                        draw_range: 0..vertices.len(),
                                        draw_mode: solstice::DrawMode::Triangles,
                                        instance_count: 1,
                                    };
                                    solstice::Renderer::draw(ctx, shader, &geometry, settings);
                                }
                                Some(indices) => {
                                    self.mesh3d.set_vertices(&vertices, 0);
                                    self.mesh3d.set_indices(&indices, 0);
                                    let mesh = self.mesh3d.unmap(ctx);
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
                    shader.send_uniform(
                        "uView",
                        solstice::shader::RawUniformValue::Mat4(camera.into()),
                    );
                    shader.send_uniform(
                        "uModel",
                        solstice::shader::RawUniformValue::Mat4(transform.into()),
                    );
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
                    shader.send_uniform(
                        "uView",
                        solstice::shader::RawUniformValue::Mat4(camera.into()),
                    );
                    shader.send_uniform(
                        "uModel",
                        solstice::shader::RawUniformValue::Mat4(transform.into()),
                    );
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

#[derive(Clone)]
pub struct Geometry<'a, V>
where
    [V]: std::borrow::ToOwned,
{
    vertices: std::borrow::Cow<'a, [V]>,
    indices: Option<std::borrow::Cow<'a, [u32]>>,
}

impl<'a, V> Geometry<'a, V>
where
    [V]: std::borrow::ToOwned,
{
    pub fn new<IV: Into<std::borrow::Cow<'a, [V]>>, II: Into<std::borrow::Cow<'a, [u32]>>>(
        vertices: IV,
        indices: Option<II>,
    ) -> Self {
        Self {
            vertices: vertices.into(),
            indices: indices.map(Into::into),
        }
    }
}

pub trait Draw<V: solstice::vertex::Vertex, G> {
    fn draw(&mut self, geometry: G);
    fn draw_with_transform<TX: Into<d3::Transform3D>>(&mut self, geometry: G, transform: TX);
    fn draw_with_color<C: Into<Color>>(&mut self, geometry: G, color: C);
    fn draw_with_color_and_transform<C: Into<Color>, TX: Into<d3::Transform3D>>(
        &mut self,
        geometry: G,
        color: C,
        transform: TX,
    );
    fn stroke(&mut self, geometry: G);
    fn stroke_with_transform<TX: Into<d3::Transform3D>>(&mut self, geometry: G, transform: TX);
    fn stroke_with_color<C: Into<Color>>(&mut self, geometry: G, color: C);
    fn stroke_with_color_and_transform<C: Into<Color>, TX: Into<d3::Transform3D>>(
        &mut self,
        geometry: G,
        color: C,
        transform: TX,
    );
    fn image<T: Texture>(&mut self, geometry: G, texture: T);
    fn image_with_color<T, C>(&mut self, geometry: G, texture: T, color: C)
    where
        T: Texture,
        C: Into<Color>;
    fn image_with_transform<T, TX>(&mut self, geometry: G, texture: T, transform: TX)
    where
        T: Texture,
        TX: Into<d3::Transform3D>;
    fn image_with_color_and_transform<T, C, TX>(
        &mut self,
        geometry: G,
        texture: T,
        color: C,
        transform: TX,
    ) where
        T: Texture,
        C: Into<Color>,
        TX: Into<d3::Transform3D>;
}

#[derive(PartialEq, Clone, Debug)]
struct TextureCache {
    ty: solstice::texture::TextureType,
    key: solstice::TextureKey,
    info: solstice::texture::TextureInfo,
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

#[derive(Clone, Debug)]
pub enum GeometryVariants<'a> {
    D2(
        std::borrow::Cow<'a, [d2::Vertex2D]>,
        Option<std::borrow::Cow<'a, [u32]>>,
    ),
    D3(
        std::borrow::Cow<'a, [d3::Vertex3D]>,
        Option<std::borrow::Cow<'a, [u32]>>,
    ),
}

#[derive(Clone, Debug)]
pub struct DrawState<T> {
    data: T,
    transform: Transform3D,
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

#[derive(Clone, Debug)]
pub struct PrintState<'a> {
    text: std::borrow::Cow<'a, str>,
    font_id: glyph_brush::FontId,
    scale: f32,
    bounds: d2::Rectangle,
}

#[derive(Clone, Debug)]
pub enum Command<'a> {
    Draw(DrawState<GeometryVariants<'a>>),
    Print(DrawState<PrintState<'a>>),
    Line(DrawState<LineState<'a>>),
    Clear(Color, Option<Canvas>),
}

#[derive(Clone, Debug, Default)]
pub struct DrawList<'a> {
    commands: Vec<Command<'a>>,
    color: Color,
    transform: Transform3D,
    camera: Transform3D,
    projection_mode: Option<Projection>,
    target: Option<Canvas>,
    shader: Option<Shader>,
}

impl<'a> DrawList<'a> {
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
            },
            transform: self.transform,
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
            transform: self.transform,
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
            transform: self.transform,
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

    pub fn set_transform<T: Into<Transform3D>>(&mut self, transform: T) {
        self.transform = transform.into();
    }

    pub fn get_transform(&self) -> Transform3D {
        self.transform
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
