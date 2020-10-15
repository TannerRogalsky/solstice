use crate::d2::vertex::Vertex2D;
use lyon_tessellation::path::math::Point;
use solstice::{mesh::MappedIndexedMesh, texture::Texture, Context};

mod shader;
mod vertex;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum DrawMode {
    Fill,
    Stroke,
}

/// An angle, in radians.
#[derive(Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Rad(pub f32);

/// An angle, in degrees.
#[derive(Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Deg(pub f32);

impl From<Rad> for Deg {
    #[inline]
    fn from(rad: Rad) -> Deg {
        Deg(rad.0 * 180.0 / std::f32::consts::PI)
    }
}

impl From<Deg> for Rad {
    #[inline]
    fn from(deg: Deg) -> Rad {
        Rad(deg.0 * std::f32::consts::PI / 180.0)
    }
}

#[derive(Copy, Clone, Default)]
pub struct Arc {
    pub arc_type: ArcType,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub angle1: Rad,
    pub angle2: Rad,
    pub segments: u32,
}

#[derive(Copy, Clone)]
pub enum ArcType {
    Pie,
    Open,
    Closed,
}

impl Default for ArcType {
    fn default() -> Self {
        ArcType::Pie
    }
}

#[derive(Copy, Clone, Default)]
pub struct Circle {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub segments: u32,
}

impl Into<Ellipse> for Circle {
    fn into(self) -> Ellipse {
        Ellipse {
            x: self.x,
            y: self.y,
            radius_x: self.radius,
            radius_y: self.radius,
            segments: self.segments,
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct Ellipse {
    pub x: f32,
    pub y: f32,
    pub radius_x: f32,
    pub radius_y: f32,
    pub segments: u32,
}

#[derive(Copy, Clone, Default)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug)]
pub enum Graphics2DError {
    ShaderError(shader::Shader2DError),
    GraphicsError(solstice::GraphicsError),
}

#[must_use]
pub struct Graphics2DLock<'a> {
    ctx: &'a mut Context,
    inner: &'a mut Graphics2D,
    color: [f32; 4],
    index_offset: usize,
    vertex_offset: usize,
}

impl Graphics2DLock<'_> {
    fn flush(&mut self) {
        let mesh = self.inner.mesh.unmap(self.ctx);
        let shader = self.inner.default_shader.activate(self.ctx);
        solstice::Renderer::draw(
            self.ctx,
            shader,
            &solstice::Geometry {
                mesh,
                draw_range: 0..self.index_offset,
                draw_mode: solstice::DrawMode::Triangles,
                instance_count: 1,
            },
            solstice::PipelineSettings {
                depth_state: None,
                ..solstice::PipelineSettings::default()
            },
        );
        self.index_offset = 0;
        self.vertex_offset = 0;
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

        if segments == 0 || angle1 == angle2 {
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

        let mut phi = angle1;
        let mut create_points = |coordinates: &mut [vertex::Vertex2D]| {
            for coordinate in coordinates.iter_mut() {
                phi += angle_shift;
                coordinate.position[0] = x + radius * phi.cos();
                coordinate.position[1] = y + radius * phi.sin();
            }
        };

        let vertices = match arc_type {
            ArcType::Pie => {
                let num_coords = segments as usize + 3;
                let mut coords = vec![vertex::Vertex2D::default(); num_coords];
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
            DrawMode::Stroke => self.stroke_polygon(&vertices[..vertices.len() - 1]),
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
            vertices.push(vertex::Vertex2D::new([x, y], self.color, [0.5, 0.5]));
        }

        for _ in 0..segments {
            phi += angle_shift;
            vertices.push(vertex::Vertex2D::new(
                [x + radius_x * phi.cos(), y + radius_y * phi.sin()],
                self.color,
                [0.5, 0.5],
            ));
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
            self.stroke_polygon(&vertices);
        }
    }
    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.bind_default_texture();
        self.stroke_polygon(&[
            vertex::Vertex2D::new([x1, y1], self.color, [0.5, 0.5]),
            vertex::Vertex2D::new([x2, y2], self.color, [0.5, 0.5]),
        ]);
    }
    pub fn rectangle(&mut self, draw_mode: DrawMode, rectangle: Rectangle) {
        self.bind_default_texture();
        let (vertices, indices) =
            rectangle_geometry(rectangle, self.color, self.vertex_offset as _);
        match draw_mode {
            DrawMode::Fill => self.fill_polygon(&vertices, &indices),
            DrawMode::Stroke => self.stroke_polygon(&vertices),
        }
    }
    pub fn image<T: Texture + Copy>(&mut self, rectangle: Rectangle, texture: T) {
        self.bind_texture(texture);
        let (vertices, indices) =
            rectangle_geometry(rectangle, self.color, self.vertex_offset as _);
        self.fill_polygon(&vertices, &indices);
    }

    fn fill_polygon(&mut self, vertices: &[vertex::Vertex2D], indices: &[u32]) {
        self.inner.mesh.set_vertices(vertices, self.vertex_offset);
        self.inner.mesh.set_indices(indices, self.index_offset);
        self.vertex_offset += vertices.len();
        self.index_offset += indices.len();
    }

    fn stroke_polygon(&mut self, vertices: &[vertex::Vertex2D]) {
        use lyon_tessellation::*;
        let mut builder = path::Builder::new();
        builder.polygon(
            &vertices
                .iter()
                .map(|v| v.position.into())
                .collect::<Vec<_>>(),
        );
        let path = builder.build();

        struct WithColor([f32; 4]);

        impl StrokeVertexConstructor<vertex::Vertex2D> for WithColor {
            fn new_vertex(
                &mut self,
                point: Point,
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
}

fn rectangle_geometry(
    rectangle: Rectangle,
    color: [f32; 4],
    offset: u32,
) -> ([Vertex2D; 4], [u32; 6]) {
    let Rectangle {
        x,
        y,
        width,
        height,
    } = rectangle;
    let vertices = [
        vertex::Vertex2D {
            position: [x, y],
            color,
            uv: [0., 0.],
        },
        vertex::Vertex2D {
            position: [x, y + height],
            color,
            uv: [0., 1.],
        },
        vertex::Vertex2D {
            position: [x + width, y + height],
            color,
            uv: [1., 1.],
        },
        vertex::Vertex2D {
            position: [x + width, y],
            color,
            uv: [1., 0.],
        },
    ];
    let indices = [
        offset + 0,
        offset + 1,
        offset + 2,
        offset + 0,
        offset + 3,
        offset + 2,
    ];
    (vertices, indices)
}

impl Drop for Graphics2DLock<'_> {
    fn drop(&mut self) {
        self.flush();
    }
}

pub struct Graphics2D {
    mesh: MappedIndexedMesh<vertex::Vertex2D, u32>,
    default_shader: shader::Shader2D,
    default_texture: solstice::image::Image,
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
        Ok(Self {
            mesh,
            default_shader,
            default_texture,
            width,
            height,
        })
    }

    pub fn start<'a>(&'a mut self, ctx: &'a mut Context) -> Graphics2DLock<'a> {
        Graphics2DLock {
            ctx,
            inner: self,
            color: [1., 1., 1., 1.],
            index_offset: 0,
            vertex_offset: 0,
        }
    }

    pub fn set_width_height(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
        self.default_shader.set_width_height(width, height)
    }
}
