mod d2;
mod d3;
mod shared;

pub use d2::*;
pub use d3::*;
pub use shared::*;
pub use solstice;

use solstice::{image::Image, mesh::MappedIndexedMesh, texture::Texture, Context};

pub struct GraphicsLock<'a> {
    ctx: &'a mut Context,
    gfx: &'a mut Graphics,
    dl: DrawList,
}

impl std::ops::Deref for GraphicsLock<'_> {
    type Target = DrawList;

    fn deref(&self) -> &Self::Target {
        &self.dl
    }
}

impl std::ops::DerefMut for GraphicsLock<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.dl
    }
}

impl std::ops::Drop for GraphicsLock<'_> {
    fn drop(&mut self) {
        self.gfx.process(self.ctx, &mut self.dl)
    }
}

pub struct Graphics {
    mesh3d: MappedIndexedMesh<Vertex3D, u32>,
    mesh2d: MappedIndexedMesh<Vertex2D, u32>,
    line_workspace: LineWorkspace,
    default_shader: Shader,
    default_texture: Image,
    text_workspace: text::Text,
    width: f32,
    height: f32,
}

impl Graphics {
    pub fn new(ctx: &mut Context, width: f32, height: f32) -> Result<Self, GraphicsError> {
        let mesh2d = MappedIndexedMesh::new(ctx, 10000, 10000)?;
        let mesh3d = MappedIndexedMesh::new(ctx, 10000, 10000)?;
        let line_workspace = LineWorkspace::new(ctx)?;
        let default_shader = Shader::new(ctx)?;
        let default_texture = create_default_texture(ctx)?;

        let text_workspace = text::Text::new(ctx)?;

        Ok(Self {
            mesh3d,
            mesh2d,
            line_workspace,
            default_shader,
            default_texture,
            text_workspace,
            width,
            height,
        })
    }

    pub fn lock<'a>(&'a mut self, ctx: &'a mut Context) -> GraphicsLock<'a> {
        GraphicsLock {
            ctx,
            gfx: self,
            dl: Default::default(),
        }
    }

    pub fn add_font(&mut self, font_data: glyph_brush::ab_glyph::FontVec) -> glyph_brush::FontId {
        self.text_workspace.add_font(font_data)
    }

    pub fn set_width_height(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn dimensions(&self) -> [f32; 2] {
        [self.width, self.height]
    }

    pub fn process(&mut self, ctx: &mut Context, draw_list: &mut DrawList) {
        for command in draw_list.commands.drain(..) {
            match command {
                Command::Draw(draw_state) => {
                    let DrawState {
                        data: (draw_mode, geometry),
                        transform,
                        camera,
                        projection_mode,
                        color,
                        texture,
                        target,
                        mut shader,
                    } = draw_state;
                    let mut cached_viewport = None;
                    let (width, height) = if let Some(canvas) = target.as_ref() {
                        let (width, height) = canvas.dimensions();
                        cached_viewport = Some(ctx.viewport());
                        ctx.set_viewport(0, 0, width as _, height as _);
                        (width, height)
                    } else {
                        (self.width, self.height)
                    };
                    match geometry {
                        GeometryVariants::D2(geometry) => {
                            let transform_verts = |mut v: Vertex2D| -> Vertex2D {
                                let [x, y] = v.position;
                                let [x, y, _] = transform.transform_point(x, y, 0.);
                                v.position = [x, y];
                                v.color = color.into();
                                v
                            };

                            let (vertices, indices) = match draw_mode {
                                DrawMode::Fill => (
                                    geometry.vertices().map(transform_verts).collect::<Vec<_>>(),
                                    geometry.indices().collect::<Vec<_>>(),
                                ),
                                DrawMode::Stroke => {
                                    // TODO: do we need to expand from indices here?
                                    stroke_polygon(
                                        geometry.vertices().map(transform_verts),
                                        color.into(),
                                    )
                                }
                            };
                            self.mesh2d.set_vertices(&vertices, 0);
                            self.mesh2d.set_indices(&indices, 0);
                            let mesh = self.mesh2d.unmap(ctx);
                            let geometry = solstice::Geometry {
                                mesh,
                                draw_range: 0..indices.len(),
                                draw_mode: solstice::DrawMode::Triangles,
                                instance_count: 1,
                            };
                            let shader = shader.as_mut().unwrap_or(&mut self.default_shader);
                            shader.set_width_height(projection_mode, width, height, false);
                            shader.send_uniform(
                                "uView",
                                solstice::shader::RawUniformValue::Mat4(camera.inner.into()),
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
                            solstice::Renderer::draw(
                                ctx,
                                shader,
                                &geometry,
                                solstice::PipelineSettings {
                                    depth_state: None,
                                    framebuffer: target.as_ref().map(|c| &c.inner),
                                    ..solstice::PipelineSettings::default()
                                },
                            );
                        }
                        GeometryVariants::D3(geometry) => {
                            let vertices = geometry.vertices().collect::<std::boxed::Box<[_]>>();
                            let indices = geometry.indices().collect::<std::boxed::Box<[_]>>();
                            self.mesh3d.set_vertices(&vertices, 0);
                            self.mesh3d.set_indices(&indices, 0);
                            let mesh = self.mesh3d.unmap(ctx);
                            let geometry = solstice::Geometry {
                                mesh,
                                draw_range: 0..indices.len(),
                                draw_mode: solstice::DrawMode::Triangles,
                                instance_count: 1,
                            };
                            let shader = shader.as_mut().unwrap_or(&mut self.default_shader);
                            shader.set_width_height(projection_mode, width, height, false);
                            shader.send_uniform(
                                "uView",
                                solstice::shader::RawUniformValue::Mat4(camera.inner.into()),
                            );
                            shader.send_uniform(
                                "uModel",
                                solstice::shader::RawUniformValue::Mat4(transform.inner.into()),
                            );
                            shader.set_color(draw_state.color);
                            match texture.as_ref() {
                                None => shader.bind_texture(&self.default_texture),
                                Some(texture) => shader.bind_texture(texture),
                            }
                            shader.activate(ctx);
                            solstice::Renderer::draw(
                                ctx,
                                shader,
                                &geometry,
                                solstice::PipelineSettings {
                                    framebuffer: target.as_ref().map(|c| &c.inner),
                                    ..solstice::PipelineSettings::default()
                                },
                            );
                        }
                    };
                    if let Some(v) = cached_viewport {
                        ctx.set_viewport(v.x(), v.y(), v.width(), v.height());
                    }
                }
                Command::Line(draw_state) => {
                    let DrawState {
                        data:
                            LineState {
                                geometry,
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
                    let verts = geometry.collect::<std::boxed::Box<[_]>>();
                    self.line_workspace.add_points(&verts);

                    let mut shader = shader.unwrap_or_else(|| self.line_workspace.shader().clone());
                    shader.set_width_height(projection_mode, self.width, self.height, false);
                    // TODO: this belongs in the above function
                    shader.send_uniform(
                        "resolution",
                        solstice::shader::RawUniformValue::Vec2([self.width, self.height].into()),
                    );
                    shader.send_uniform(
                        "uView",
                        solstice::shader::RawUniformValue::Mat4(camera.inner.into()),
                    );
                    shader.send_uniform(
                        "uModel",
                        solstice::shader::RawUniformValue::Mat4(transform.inner.into()),
                    );
                    match texture.as_ref() {
                        None => shader.bind_texture(&self.default_texture),
                        Some(texture) => shader.bind_texture(texture),
                    }
                    shader.set_color(color);
                    shader.activate(ctx);

                    let geometry = self.line_workspace.geometry(ctx);

                    let depth_state = if depth_buffer {
                        Some(solstice::DepthState::default())
                    } else {
                        None
                    };

                    solstice::Renderer::draw(
                        ctx,
                        &shader,
                        &geometry,
                        solstice::PipelineSettings {
                            depth_state,
                            framebuffer: target.as_ref().map(|c| &c.inner),
                            ..solstice::PipelineSettings::default()
                        },
                    )
                }
                Command::Clear(color, target) => {
                    solstice::Renderer::clear(
                        ctx,
                        solstice::ClearSettings {
                            color: Some(color.into()),
                            target: target.as_ref().map(|c| &c.inner),
                            ..solstice::ClearSettings::default()
                        },
                    );
                }
            }
        }
    }
}

fn stroke_polygon<P, I>(vertices: I, color: [f32; 4]) -> (Vec<Vertex2D>, Vec<u32>)
where
    P: Into<lyon_tessellation::math::Point>,
    I: IntoIterator<Item = P>,
{
    use lyon_tessellation::*;
    let mut builder = path::Builder::new();
    builder.polygon(
        &vertices
            .into_iter()
            .map(Into::into)
            .collect::<std::boxed::Box<[_]>>(),
    );
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

pub trait Geometry<V: solstice::vertex::Vertex>: std::fmt::Debug {
    type Vertices: Iterator<Item = V>;
    type Indices: Iterator<Item = u32>;

    fn vertices(&self) -> Self::Vertices;
    fn indices(&self) -> Self::Indices;
}

pub trait BoxedGeometry<'a, V, I>: dyn_clone::DynClone + std::fmt::Debug
where
    V: solstice::vertex::Vertex,
    I: solstice::mesh::Index,
{
    fn vertices(&self) -> std::boxed::Box<dyn Iterator<Item = V> + 'a>;
    fn indices(&self) -> std::boxed::Box<dyn Iterator<Item = I> + 'a>;
}

pub trait Draw<V: solstice::vertex::Vertex, G: Geometry<V> + Clone + 'static> {
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DrawMode {
    Fill,
    Stroke,
}

#[derive(Clone, Debug)]
pub enum GeometryVariants {
    D2(std::boxed::Box<dyn BoxedGeometry<'static, d2::Vertex2D, u32>>),
    D3(std::boxed::Box<dyn BoxedGeometry<'static, d3::Vertex3D, u32>>),
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

trait VertexGeometry:
    Iterator<Item = LineVertex> + dyn_clone::DynClone + std::fmt::Debug + 'static
{
}
impl<T: Iterator<Item = LineVertex> + dyn_clone::DynClone + std::fmt::Debug + 'static>
    VertexGeometry for T
{
}
dyn_clone::clone_trait_object!(VertexGeometry);

// Maybe DrawState just be generic on it's geometry
// That would mean we have less assurance about the type
// How would that change the processing?
#[derive(Clone, Debug)]
pub struct LineState {
    geometry: std::boxed::Box<dyn VertexGeometry>,
    depth_buffer: bool,
}

#[derive(Clone, Debug)]
pub enum Command {
    Draw(DrawState<(DrawMode, GeometryVariants)>),
    Line(DrawState<LineState>),
    Clear(Color, Option<Canvas>),
}

#[derive(Clone, Debug, Default)]
pub struct DrawList {
    commands: Vec<Command>,
    color: Color,
    transform: Transform3D,
    camera: Transform3D,
    projection_mode: Option<Projection>,
    target: Option<Canvas>,
    shader: Option<Shader>,
}

impl DrawList {
    pub fn clear<C: Into<Color>>(&mut self, color: C) {
        let command = Command::Clear(color.into(), self.target.clone());
        self.commands.push(command)
    }

    pub fn line_2d<G: Iterator<Item = LineVertex> + Clone + std::fmt::Debug + 'static>(
        &mut self,
        points: G,
    ) {
        let command = Command::Line(DrawState {
            data: LineState {
                geometry: std::boxed::Box::new(points),
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

    pub fn line_3d<G: Iterator<Item = LineVertex> + Clone + std::fmt::Debug + 'static>(
        &mut self,
        points: G,
    ) {
        let command = Command::Line(DrawState {
            data: LineState {
                geometry: std::boxed::Box::new(points),
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

struct PerlinSampler {
    width: usize,
    height: usize,
    gradients: Vec<f32>,
}

impl PerlinSampler {
    pub fn new<R: rand::Rng>(width: usize, height: usize, mut rng: R) -> Self {
        let mut gradients = Vec::with_capacity(width * height * 2);
        const TAU: f32 = std::f32::consts::PI * 2.;
        for _i in (0..(width * height * 2)).step_by(2) {
            let phi = rng.gen::<f32>() * TAU;
            let (x, y) = phi.sin_cos();
            gradients.push(x);
            gradients.push(y);
        }
        Self {
            width,
            height,
            gradients,
        }
    }

    pub fn dot(&self, x_cell: usize, y_cell: usize, vx: f32, vy: f32) -> f32 {
        let offset = (x_cell + y_cell * self.width) * 2;
        let wx = self.gradients[offset];
        let wy = self.gradients[offset + 1];
        wx * vx + wy * vy
    }

    pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + t * (b - a)
    }

    pub fn s_curve(t: f32) -> f32 {
        t * t * (3. - 2. * t)
    }

    pub fn get(&self, x: f32, y: f32) -> f32 {
        let x_cell = x.trunc() as usize;
        let y_cell = y.trunc() as usize;
        let x_fract = x.fract();
        let y_fract = y.fract();

        let x0 = x_cell;
        let y0 = y_cell;
        let x1 = if x_cell == (self.width - 1) {
            0
        } else {
            x_cell + 1
        };
        let y1 = if y_cell == (self.height - 1) {
            0
        } else {
            y_cell + 1
        };

        let v00 = self.dot(x0, y0, x_fract, y_fract);
        let v10 = self.dot(x1, y0, x_fract - 1., y_fract);
        let v01 = self.dot(x0, y1, x_fract, y_fract - 1.);
        let v11 = self.dot(x1, y1, x_fract - 1., y_fract - 1.);

        let vx0 = Self::lerp(v00, v10, Self::s_curve(x_fract));
        let vx1 = Self::lerp(v01, v11, Self::s_curve(x_fract));

        return Self::lerp(vx0, vx1, Self::s_curve(y_fract));
    }
}

// spec.randseed = document.getElementById("randseed").value;
// spec.period = document.getElementById("period").value;
// spec.levels = document.getElementById("numLevels").value;
// spec.atten = document.getElementById("atten").value;
// spec.absolute = document.getElementById("absolute").checked;
// spec.color = document.getElementById("noiseColor").checked;
// spec.alpha = document.getElementById("noiseAlpha").checked;
pub struct PerlinTextureSettings<R> {
    pub rng: R,
    pub width: usize,
    pub height: usize,
    pub period: u32,
    pub levels: u32,
    pub attenuation: f32,
}

pub fn create_perlin_texture<R: rand::Rng>(
    gl: &mut solstice::Context,
    settings: PerlinTextureSettings<R>,
) -> ImageResult {
    let PerlinTextureSettings {
        mut rng,
        width,
        height,
        period,
        levels,
        attenuation,
    } = settings;
    let num_channels = 3;
    let mut raster = vec![0f32; width * height * num_channels];
    for channel in 0..num_channels {
        let mut local_period_inv = 1. / period as f32;
        let mut freq_inv = 1f32;
        let mut atten = 1.;
        let mut weight = 0f32;

        for _level in 0..levels {
            let sampler = PerlinSampler::new(
                (width as f32 * local_period_inv).ceil() as usize,
                (height as f32 * local_period_inv).ceil() as usize,
                &mut rng,
            );
            for y in 0..height {
                for x in 0..width {
                    let val = sampler.get(x as f32 * local_period_inv, y as f32 * local_period_inv);
                    raster[(x + y * width) * num_channels + channel] += val * freq_inv.powf(atten);
                }
            }
            weight += freq_inv.powf(atten);
            freq_inv *= 0.5;
            local_period_inv *= 2.;
            atten *= attenuation;
        }

        let weight_inv = 1. / weight;
        for y in 0..height {
            for x in 0..width {
                raster[(x + y * width) * num_channels + channel] *= weight_inv;
            }
        }
    }

    let mut bytes = vec![0u8; width * height * num_channels];
    for (p, f) in bytes.iter_mut().zip(raster.into_iter()) {
        *p = (((f + 1.) / 2.) * 255.) as u8;
    }

    use solstice::image::*;
    use solstice::texture::*;
    Image::with_data(
        gl,
        TextureType::Tex2D,
        solstice::PixelFormat::RGB8,
        width as u32,
        height as u32,
        bytes.as_slice(),
        Settings {
            mipmaps: false,
            filter: FilterMode::Linear,
            wrap: WrapMode::Repeat,
            ..Settings::default()
        },
    )
}
