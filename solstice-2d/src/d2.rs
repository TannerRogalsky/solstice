use solstice::{mesh::MappedIndexedMesh, texture::Texture, Context};

mod canvas;
mod shader;
mod shapes;
mod text;
mod transforms;
mod vertex;

pub use canvas::Canvas;
pub use glyph_brush::{ab_glyph::FontVec, FontId};
pub use shader::Shader2D;
pub use shapes::*;
pub use transforms::*;
pub use vertex::{Point, Vertex2D};

#[derive(Copy, Clone, Eq, PartialEq)]
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
pub struct Graphics2DLock<'a, 's> {
    ctx: &'a mut Context,
    inner: &'a mut Graphics2D,
    color: [f32; 4],
    index_offset: usize,
    vertex_offset: usize,
    pub transforms: Transforms,

    active_shader: Option<&'s mut Shader2D>,
    active_canvas: Option<&'s Canvas>,
}

impl<'a, 's> Graphics2DLock<'a, 's> {
    fn flush(&mut self) {
        let mesh = self.inner.mesh.unmap(self.ctx);

        let geometry = solstice::Geometry {
            mesh,
            draw_range: 0..self.index_offset,
            draw_mode: solstice::DrawMode::Triangles,
            instance_count: 1,
        };

        let shader = match self.active_shader.as_mut() {
            None => &mut self.inner.default_shader,
            Some(shader) => *shader,
        };

        let viewport = self.ctx.viewport();
        match self.active_canvas {
            None => {
                shader.set_width_height(self.inner.width, self.inner.height, false);
            }
            Some(canvas) => {
                let (width, height) = canvas.dimensions();
                // TODO: this sort of thing might be better handled with a whole state stack push/pop
                self.ctx.set_viewport(0, 0, width as _, height as _);
                shader.set_width_height(width, height, true);
            }
        }

        shader.activate(self.ctx);
        solstice::Renderer::draw(
            self.ctx,
            shader,
            &geometry,
            solstice::PipelineSettings {
                depth_state: None,
                framebuffer: self.active_canvas.map(|c| &c.inner),
                ..solstice::PipelineSettings::default()
            },
        );

        // rollback the viewport change
        if self.active_canvas.is_some() {
            self.ctx.set_viewport(
                viewport.x(),
                viewport.y(),
                viewport.width(),
                viewport.height(),
            );
        }

        self.index_offset = 0;
        self.vertex_offset = 0;
    }

    pub fn clear<C: Into<[f32; 4]>>(&mut self, color: C) {
        let [red, green, blue, alpha] = color.into();
        let color = solstice::Color {
            red,
            blue,
            green,
            alpha,
        };
        solstice::Renderer::clear(
            self.ctx,
            solstice::ClearSettings {
                color: Some(color.into()),
                depth: None,
                stencil: None,
                target: self.active_canvas.map(|c| &c.inner),
            },
        )
    }

    pub fn set_shader(&mut self, shader: &'s mut Shader2D) {
        self.flush();
        self.active_shader.replace(shader);
    }

    pub fn remove_active_shader(&mut self) {
        self.flush();
        self.active_shader.take();
    }

    pub fn set_canvas(&mut self, canvas: &'s Canvas) {
        self.flush();
        self.active_canvas.replace(canvas);
    }

    pub fn unset_canvas(&mut self) {
        self.flush();
        self.active_canvas.take();
    }

    fn shader(&self) -> &Shader2D {
        self.active_shader
            .as_deref()
            .unwrap_or(&self.inner.default_shader)
    }

    fn bind_default_texture(&mut self) {
        let texture = &self.inner.default_texture;
        if !self.shader().is_bound(texture) {
            self.flush();
            let texture = &self.inner.default_texture;
            match &mut self.active_shader {
                Some(shader) => shader.bind_texture(texture),
                None => self.inner.default_shader.bind_texture(texture),
            }
        }
    }

    fn bind_texture<T: Texture + Copy>(&mut self, texture: T) {
        if !self.shader().is_bound(texture) {
            self.flush();
            match &mut self.active_shader {
                Some(shader) => shader.bind_texture(texture),
                None => self.inner.default_shader.bind_texture(texture),
            }
        }
    }

    pub fn set_color<C: Into<[f32; 4]>>(&mut self, color: C) {
        self.color = color.into();
    }

    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.bind_default_texture();
        let (x1, y1) = self.transforms.current().transform_point(x1, y1);
        let (x2, y2) = self.transforms.current().transform_point(x2, y2);
        self.stroke_polygon([Point::new(x1, y1), Point::new(x2, y2)].iter(), self.color)
    }
    pub fn lines<P: Into<Point>, I: IntoIterator<Item = P>>(&mut self, points: I) {
        let transform = *self.transforms.current();
        let points = points.into_iter().map(|p| {
            let point: Point = p.into();
            Into::<lyon_tessellation::math::Point>::into(
                transform.transform_point(point.x, point.y),
            )
        });
        self.stroke_polygon(points, self.color);
    }

    pub fn image<G, T>(&mut self, geometry: G, texture: T)
    where
        G: Geometry,
        T: Texture + Copy,
    {
        self.bind_texture(texture);
        self.inner_draw(
            DrawMode::Fill,
            geometry,
            *self.transforms.current(),
            self.color,
        );
    }

    pub fn image_with_transform<G, T>(&mut self, geometry: G, texture: T, transform: Transform)
    where
        G: Geometry,
        T: Texture + Copy,
    {
        self.bind_texture(texture);
        self.inner_draw(DrawMode::Fill, geometry, transform, self.color)
    }

    pub fn image_with_color<G, T, C>(&mut self, geometry: G, texture: T, color: C)
    where
        G: Geometry,
        T: Texture + Copy,
        C: Into<[f32; 4]>,
    {
        self.bind_texture(texture);
        self.inner_draw(
            DrawMode::Fill,
            geometry,
            *self.transforms.current(),
            color.into(),
        );
    }

    pub fn image_with_transform_and_color<G, T, C>(
        &mut self,
        geometry: G,
        texture: T,
        transform: Transform,
        color: C,
    ) where
        G: Geometry,
        T: Texture + Copy,
        C: Into<[f32; 4]>,
    {
        self.bind_texture(texture);
        self.inner_draw(DrawMode::Fill, geometry, transform, color.into());
    }

    pub fn draw<G: Geometry>(&mut self, draw_mode: DrawMode, geometry: G) {
        self.bind_default_texture();
        self.inner_draw(draw_mode, geometry, *self.transforms.current(), self.color);
    }

    pub fn draw_with_transform<G>(&mut self, draw_mode: DrawMode, geometry: G, transform: Transform)
    where
        G: Geometry,
    {
        self.bind_default_texture();
        self.inner_draw(draw_mode, geometry, transform, self.color)
    }

    pub fn draw_with_color<G, C>(&mut self, draw_mode: DrawMode, geometry: G, color: C)
    where
        G: Geometry,
        C: Into<[f32; 4]>,
    {
        self.bind_default_texture();
        self.inner_draw(
            draw_mode,
            geometry,
            *self.transforms.current(),
            color.into(),
        );
    }

    pub fn draw_with_transform_and_color<G, C>(
        &mut self,
        draw_mode: DrawMode,
        geometry: G,
        transform: Transform,
        color: C,
    ) where
        G: Geometry,
        C: Into<[f32; 4]>,
    {
        self.bind_default_texture();
        self.inner_draw(draw_mode, geometry, transform, color.into());
    }

    fn inner_draw<G: Geometry>(
        &mut self,
        draw_mode: DrawMode,
        geometry: G,
        transform: Transform,
        color: [f32; 4],
    ) {
        let transform_vertex = move |v: Vertex2D| {
            let (x, y) = transform.transform_point(v.position[0], v.position[1]);
            Vertex2D {
                position: [x, y],
                color,
                ..v
            }
        };
        match draw_mode {
            DrawMode::Fill => {
                let vertices = geometry
                    .vertices()
                    .map(transform_vertex)
                    .collect::<Box<_>>();
                let indices = geometry
                    .indices()
                    .map(|i| self.vertex_offset as u32 + i)
                    .collect::<Box<_>>();
                self.fill_polygon(&vertices, &indices)
            }
            DrawMode::Stroke => self.stroke_polygon(
                geometry
                    .vertices()
                    .map(transform_vertex)
                    .map(Into::<lyon_tessellation::math::Point>::into),
                color,
            ),
        }
    }

    fn fill_polygon(&mut self, vertices: &[Vertex2D], indices: &[u32]) {
        self.buffer_geometry(vertices, indices)
    }

    fn stroke_polygon<P, I>(&mut self, vertices: I, color: [f32; 4])
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

        let indices = buffers
            .indices
            .iter()
            .map(|i| self.vertex_offset as u32 + *i)
            .collect::<Vec<_>>();

        self.buffer_geometry(&buffers.vertices, &indices);
    }

    fn buffer_geometry(&mut self, vertices: &[Vertex2D], indices: &[u32]) {
        if self.vertex_offset + vertices.len() > self.inner.mesh.vertex_capacity()
            || self.index_offset + indices.len() > self.inner.mesh.index_capacity()
        {
            // because we're flushing, all the index offsets are going to be wrong
            // we could avoid this if we knew before that we were going overflow the mesh
            // this is mostly straightforward except for the stroke tesselation so
            // TODO: when lyon is removed, figure out sizes and do the overflow check earlier
            let indices = indices
                .iter()
                .map(|i| *i - self.vertex_offset as u32)
                .collect::<Vec<_>>();
            self.flush();
            self.inner.mesh.set_vertices(vertices, self.vertex_offset);
            self.inner.mesh.set_indices(&indices, self.index_offset);
        } else {
            self.inner.mesh.set_vertices(vertices, self.vertex_offset);
            self.inner.mesh.set_indices(indices, self.index_offset);
        }
        self.vertex_offset += vertices.len();
        self.index_offset += indices.len();
    }

    pub fn print<S: AsRef<str>>(
        &mut self,
        font: glyph_brush::FontId,
        text: S,
        x: f32,
        y: f32,
        scale: f32,
    ) {
        self.flush();
        let bounds = Rectangle {
            x,
            y,
            width: self.inner.width - x,
            height: self.inner.height - y,
        };
        let text = glyph_brush::Text::new(text.as_ref())
            .with_color(self.color)
            .with_font_id(font)
            .with_scale(scale);
        self.inner.text_workspace.set_text(text, bounds, self.ctx);

        let invert_y = self.active_canvas.is_some();
        let shader = &mut self.inner.text_shader;
        shader.set_width_height(self.inner.width, self.inner.height, invert_y);
        shader.bind_texture(self.inner.text_workspace.texture());
        shader.activate(self.ctx);
        let geometry = self.inner.text_workspace.geometry(self.ctx);
        solstice::Renderer::draw(
            self.ctx,
            shader,
            &geometry,
            solstice::PipelineSettings {
                depth_state: None,
                ..solstice::PipelineSettings::default()
            },
        );
    }
}

impl Drop for Graphics2DLock<'_, '_> {
    fn drop(&mut self) {
        self.flush();
    }
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
            color: [1., 1., 1., 1.],
            index_offset: 0,
            vertex_offset: 0,
            transforms: Default::default(),
            active_shader: None,
            active_canvas: None,
        }
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

pub trait Geometry {
    type Vertices: Iterator<Item = Vertex2D>;
    type Indices: Iterator<Item = u32>;

    fn vertices(&self) -> Self::Vertices;
    fn indices(&self) -> Self::Indices;
}

pub trait SimpleConvexGeometry {
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
