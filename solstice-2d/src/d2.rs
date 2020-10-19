use crate::d2::vertex::Vertex2D;
use solstice::{mesh::MappedIndexedMesh, texture::Texture, Context};

mod shader;
mod shapes;
mod text;
mod transforms;
mod vertex;

use vertex::Point;

pub use glyph_brush::FontId;
pub use shader::Shader2D;
pub use shapes::*;
pub use transforms::*;

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

fn to_points<'a>(vertices: &'a [Vertex2D]) -> impl Iterator<Item = Point> + 'a {
    vertices.iter().map(Vertex2D::position).map(Into::into)
}

#[must_use]
pub struct Graphics2DLock<'a, 's> {
    ctx: &'a mut Context,
    inner: &'a mut Graphics2D,
    color: [f32; 4],
    index_offset: usize,
    vertex_offset: usize,
    pub transforms: Transforms,

    active_shader: Option<&'s mut shader::Shader2D>,
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
        shader.set_width_height(self.inner.width, self.inner.height);
        shader.activate(self.ctx);
        solstice::Renderer::draw(
            self.ctx,
            shader,
            &geometry,
            solstice::PipelineSettings {
                depth_state: None,
                ..solstice::PipelineSettings::default()
            },
        );

        self.index_offset = 0;
        self.vertex_offset = 0;
    }

    pub fn set_shader(&mut self, shader: &'s mut shader::Shader2D) {
        self.flush();
        self.active_shader.replace(shader);
    }

    pub fn remove_active_shader(&mut self) {
        self.flush();
        self.active_shader.take();
    }

    fn bind_default_texture(&mut self) {
        let texture = &self.inner.default_texture;
        if !self.inner.default_shader.is_bound(texture) {
            self.flush();
            let texture = &self.inner.default_texture;
            self.inner.default_shader.bind_texture(texture);
        }
    }

    fn bind_texture<T: Texture + Copy>(&mut self, texture: T) {
        if !self.inner.default_shader.is_bound(texture) {
            self.flush();
            self.inner.default_shader.bind_texture(texture);
        }
    }

    pub fn set_color<C: Into<[f32; 4]>>(&mut self, color: C) {
        self.color = color.into();
    }

    pub fn arc(&mut self, draw_mode: DrawMode, arc: Arc) {
        self.bind_default_texture();
        let Arc {
            arc_type,
            x,
            y,
            radius,
            angle1,
            angle2,
            segments,
        } = arc;
        let (angle1, angle2) = (angle1.0, angle2.0);

        if segments == 0 || (angle1 - angle2).abs() < std::f32::EPSILON {
            return;
        }

        const TWO_PI: f32 = std::f32::consts::PI * 2.;
        if (angle1 - angle2).abs() >= TWO_PI {
            return self.circle(
                draw_mode,
                Circle {
                    x,
                    y,
                    radius,
                    segments,
                },
            );
        }

        let angle_shift = (angle2 - angle1) / segments as f32;
        if angle_shift == 0. {
            return; // bail on precision fail
        }

        let transform = self.transforms.current();
        let mut create_points = {
            let mut phi = angle1;
            move |coordinates: &mut [vertex::Vertex2D]| {
                for coordinate in coordinates.iter_mut() {
                    phi += angle_shift;
                    let x = x + radius * phi.cos();
                    let y = y + radius * phi.sin();
                    let (x, y) = transform.transform_point(x, y);
                    coordinate.position[0] = x;
                    coordinate.position[1] = y;
                }
            }
        };

        let vertices = match arc_type {
            ArcType::Pie => {
                let num_coords = segments as usize + 3;
                let mut coords = vec![vertex::Vertex2D::default(); num_coords];
                let (x, y) = transform.transform_point(x, y);
                coords[0] = vertex::Vertex2D {
                    position: [x, y],
                    ..vertex::Vertex2D::default()
                };
                create_points(&mut coords[1..]);
                coords
            }
            ArcType::Open => {
                let num_coords = segments as usize + 1;
                let mut coords = vec![vertex::Vertex2D::default(); num_coords];
                create_points(&mut coords);
                coords
            }
            ArcType::Closed => {
                let num_coords = segments as usize + 2;
                let mut coords = vec![vertex::Vertex2D::default(); num_coords];
                create_points(&mut coords);
                coords[num_coords - 1] = coords[0];
                coords
            }
        };

        match draw_mode {
            DrawMode::Fill => {
                let mut indices = Vec::with_capacity(vertices.len() * 3);
                let offset = self.vertex_offset as u32;
                for i in 1..(vertices.len() - 2) as u32 {
                    indices.push(offset);
                    indices.push(offset + i);
                    indices.push(offset + i + 1);
                }
                self.fill_polygon(&vertices, &indices);
            }
            DrawMode::Stroke => self.stroke_polygon(to_points(&vertices[..vertices.len() - 1])),
        }
    }
    pub fn circle(&mut self, draw_mode: DrawMode, circle: Circle) {
        self.ellipse(draw_mode, circle.into());
    }
    pub fn ellipse(&mut self, draw_mode: DrawMode, ellipse: Ellipse) {
        let Ellipse {
            x,
            y,
            radius_x,
            radius_y,
            segments,
        } = ellipse;
        const TWO_PI: f32 = std::f32::consts::PI * 2.;
        if segments == 0 {
            return;
        }
        let angle_shift = TWO_PI / segments as f32;
        let mut phi = 0.;

        let extra_segments = 1 + match draw_mode {
            DrawMode::Fill => 1,
            DrawMode::Stroke => 0,
        };

        let mut vertices = Vec::with_capacity((segments + extra_segments) as usize);

        if draw_mode == DrawMode::Fill {
            let (x, y) = self.transforms.current().transform_point(x, y);
            vertices.push(vertex::Vertex2D::new([x, y], self.color, [0.5, 0.5]));
        }

        let transform = self.transforms.current();
        for _ in 0..segments {
            phi += angle_shift;
            let (x, y) = (x + radius_x * phi.cos(), y + radius_y * phi.sin());
            let (x, y) = transform.transform_point(x, y);
            vertices.push(vertex::Vertex2D::new([x, y], self.color, [0.5, 0.5]));
        }
        vertices.push(vertices[extra_segments as usize - 1]);

        self.bind_default_texture();

        if draw_mode == DrawMode::Fill {
            let mut indices = Vec::with_capacity(segments as usize * 3);
            let offset = self.vertex_offset as u32;
            for i in 1..=segments {
                indices.push(offset);
                indices.push(offset + i);
                indices.push(offset + i + 1);
            }
            self.fill_polygon(&vertices, &indices);
        } else {
            self.stroke_polygon(to_points(&vertices));
        }
    }
    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.bind_default_texture();
        let (x1, y1) = self.transforms.current().transform_point(x1, y1);
        let (x2, y2) = self.transforms.current().transform_point(x2, y2);
        self.stroke_polygon([Point::new(x1, y1), Point::new(x2, y2)].iter())
    }
    pub fn lines<P: Into<Point>, I: IntoIterator<Item = P>>(&mut self, points: I) {
        let transform = *self.transforms.current();
        let points = points.into_iter().map(|p| {
            let point: Point = p.into();
            Into::<lyon_tessellation::math::Point>::into(
                transform.transform_point(point.x, point.y),
            )
        });
        self.stroke_polygon(points);
    }
    pub fn rectangle(&mut self, draw_mode: DrawMode, rectangle: Rectangle) {
        self.bind_default_texture();
        let (vertices, indices) = self.rectangle_geometry(rectangle);
        match draw_mode {
            DrawMode::Fill => self.fill_polygon(&vertices, &indices),
            DrawMode::Stroke => self.stroke_polygon(to_points(&vertices)),
        }
    }
    pub fn image<T: Texture + Copy>(&mut self, rectangle: Rectangle, texture: T) {
        self.bind_texture(texture);
        let (vertices, indices) = self.rectangle_geometry(rectangle);
        self.fill_polygon(&vertices, &indices);
    }

    fn fill_polygon(&mut self, vertices: &[vertex::Vertex2D], indices: &[u32]) {
        self.inner.mesh.set_vertices(vertices, self.vertex_offset);
        self.inner.mesh.set_indices(indices, self.index_offset);
        self.vertex_offset += vertices.len();
        self.index_offset += indices.len();
    }

    fn stroke_polygon<P, I>(&mut self, vertices: I)
    where
        P: Into<lyon_tessellation::math::Point>,
        I: IntoIterator<Item = P>,
    {
        use lyon_tessellation::*;
        let mut builder = path::Builder::new();
        builder.polygon(&vertices.into_iter().map(Into::into).collect::<Box<[_]>>());
        let path = builder.build();

        struct WithColor([f32; 4]);

        impl StrokeVertexConstructor<vertex::Vertex2D> for WithColor {
            fn new_vertex(
                &mut self,
                point: lyon_tessellation::math::Point,
                attributes: StrokeAttributes<'_, '_>,
            ) -> Vertex2D {
                vertex::Vertex2D {
                    position: [point.x, point.y],
                    color: self.0,
                    uv: attributes.normal().into(),
                }
            }
        }

        let mut buffers: VertexBuffers<vertex::Vertex2D, u32> = VertexBuffers::new();
        {
            let mut tessellator = StrokeTessellator::new();
            tessellator
                .tessellate(
                    &path,
                    &StrokeOptions::default().with_line_width(5.),
                    &mut BuffersBuilder::new(&mut buffers, WithColor(self.color)),
                )
                .unwrap();
        }

        let indices = buffers
            .indices
            .iter()
            .map(|i| self.vertex_offset as u32 + *i)
            .collect::<Vec<_>>();

        self.inner
            .mesh
            .set_vertices(&buffers.vertices, self.vertex_offset);
        self.inner.mesh.set_indices(&indices, self.index_offset);
        self.vertex_offset += buffers.vertices.len();
        self.index_offset += buffers.indices.len();
    }
    fn rectangle_geometry(&self, rectangle: Rectangle) -> ([Vertex2D; 4], [u32; 6]) {
        let offset = self.vertex_offset as u32;
        let color = self.color;
        let Rectangle {
            x,
            y,
            width,
            height,
        } = rectangle;
        let transform = self.transforms.current();
        let (x1, y1) = transform.transform_point(x, y);
        let (x2, y2) = transform.transform_point(x + width, y + height);

        let vertices = [
            vertex::Vertex2D {
                position: [x1, y1],
                color,
                uv: [0., 0.],
            },
            vertex::Vertex2D {
                position: [x1, y2],
                color,
                uv: [0., 1.],
            },
            vertex::Vertex2D {
                position: [x2, y2],
                color,
                uv: [1., 1.],
            },
            vertex::Vertex2D {
                position: [x2, y1],
                color,
                uv: [1., 0.],
            },
        ];
        let indices = [
            offset,
            offset + 1,
            offset + 2,
            offset,
            offset + 3,
            offset + 2,
        ];
        (vertices, indices)
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
        let text_workspace = &mut self.inner.text_workspace;
        text_workspace.set_text(text, bounds, self.ctx);

        let shader = &mut self.inner.default_shader;
        shader.bind_texture(text_workspace.texture());
        shader.activate(self.ctx);
        let geometry = text_workspace.geometry(self.ctx);
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
    mesh: MappedIndexedMesh<vertex::Vertex2D, u32>,
    default_shader: shader::Shader2D,
    default_texture: solstice::image::Image,
    text_workspace: text::Text,
    width: f32,
    height: f32,
}

impl Graphics2D {
    pub fn new(ctx: &mut Context, width: f32, height: f32) -> Result<Self, Graphics2DError> {
        let mesh =
            MappedIndexedMesh::new(ctx, 10000, 10000).map_err(Graphics2DError::GraphicsError)?;
        let default_shader =
            shader::Shader2D::new(ctx, width, height).map_err(Graphics2DError::ShaderError)?;
        let default_texture = super::create_default_texture(ctx);
        let text_workspace = text::Text::new(ctx).map_err(Graphics2DError::GraphicsError)?;
        Ok(Self {
            mesh,
            default_shader,
            default_texture,
            text_workspace,
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
        }
    }

    pub fn add_font(&mut self, font_data: glyph_brush::ab_glyph::FontVec) -> glyph_brush::FontId {
        self.text_workspace.add_font(font_data)
    }

    pub fn set_width_height(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        self.default_shader.set_width_height(width, height)
    }
}
