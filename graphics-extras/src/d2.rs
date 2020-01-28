use graphics::mesh::Mesh;
use graphics::Context;

use std::cell::RefCell;
use std::rc::Rc;

mod shader;
mod vertex;

pub enum DrawMode {
    Fill,
    Stroke,
}

/// An angle, in radians.
///
/// This type is marked as `#[repr(C)]`.
#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Rad(pub f32);

/// An angle, in degrees.
///
/// This type is marked as `#[repr(C)]`.
#[repr(C)]
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

pub struct Graphics2D {
    gfx: Rc<RefCell<Context>>,
    mesh: Mesh<vertex::Vertex2D>,
    default_shader: shader::Shader2D,
    default_texture: graphics::image::Image,
}

impl Graphics2D {
    pub fn new(gfx: Rc<RefCell<Context>>, width: f32, height: f32) -> Result<Self, String> {
        let mesh = Mesh::with_capacities(&mut gfx.borrow_mut(), 1000, 0);
        let default_shader = shader::Shader2D::new(gfx.clone(), width, height)?;
        let default_texture = super::create_default_texture(&mut gfx.borrow_mut());
        Ok(Self {
            gfx,
            mesh,
            default_shader,
            default_texture,
        })
    }

    pub fn set_width_height(&mut self, width: f32, height: f32) {
        self.gfx
            .borrow_mut()
            .set_viewport(0, 0, width as i32, height as i32);
        self.default_shader.set_width_height(width, height)
    }

    pub fn set_color(&mut self, color: mint::Vector4<f32>) {
        self.default_shader.set_color(color)
    }

    pub fn arc(&mut self, draw_mode: DrawMode, arc: Arc) {
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

        let coords = match arc_type {
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

        self.default_shader.bind_texture(&self.default_texture);
        self.polygon(draw_mode, &coords, true);
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

        match draw_mode {
            DrawMode::Fill => {
                vertices.push(vertex::Vertex2D::new([x, y], [1., 1., 1., 1.], [0.5, 0.5]));
            }
            DrawMode::Stroke => (),
        }

        for _ in 0..segments {
            phi += angle_shift;
            vertices.push(vertex::Vertex2D::new(
                [x + radius_x * phi.cos(), y + radius_y * phi.sin()],
                [1., 1., 1., 1.],
                [0.5, 0.5],
            ));
        }

        vertices.push(vertices[extra_segments as usize - 1]);

        self.default_shader.bind_texture(&self.default_texture);
        self.polygon(draw_mode, &vertices, false);
    }
    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.mesh.set_vertices(
            &[
                vertex::Vertex2D::new([x1, y1], [1., 1., 1., 1.], [0.5, 0.5]),
                vertex::Vertex2D::new([x2, y2], [1., 1., 1., 1.], [0.5, 0.5]),
            ],
            0,
        );
        self.mesh.set_draw_range(Some(0..2));
        self.mesh.set_draw_mode(graphics::DrawMode::Lines);
        self.mesh.draw(&mut self.gfx.borrow_mut());
    }
    pub fn point(&mut self, x: f32, y: f32) {
        self.mesh.set_vertices(
            &[vertex::Vertex2D::new([x, y], [1., 1., 1., 1.], [0.5, 0.5])],
            0,
        );
        self.mesh.set_draw_range(Some(0..1));
        self.mesh.set_draw_mode(graphics::DrawMode::Points);
        self.mesh.draw(&mut self.gfx.borrow_mut());
    }
    pub fn rectangle(&mut self, draw_mode: DrawMode, rectangle: Rectangle) {
        let Rectangle {
            x,
            y,
            width,
            height,
        } = rectangle;
        self.default_shader.bind_texture(&self.default_texture);
        let vertices = [
            vertex::Vertex2D {
                position: [x, y],
                ..vertex::Vertex2D::default()
            },
            vertex::Vertex2D {
                position: [x, y + height],
                ..vertex::Vertex2D::default()
            },
            vertex::Vertex2D {
                position: [x + width, y + height],
                ..vertex::Vertex2D::default()
            },
            vertex::Vertex2D {
                position: [x + width, y],
                ..vertex::Vertex2D::default()
            },
            vertex::Vertex2D {
                position: [x, y],
                ..vertex::Vertex2D::default()
            },
        ];
        self.polygon(draw_mode, &vertices, true);
    }
    pub fn image<T>(&mut self, rectangle: Rectangle, texture: T)
    where
        T: graphics::texture::Texture,
    {
        let Rectangle {
            x,
            y,
            width,
            height,
        } = rectangle;
        self.default_shader.bind_texture(texture);
        let vertices = [
            vertex::Vertex2D {
                position: [x, y],
                uv: [0., 0.],
                ..vertex::Vertex2D::default()
            },
            vertex::Vertex2D {
                position: [x, y + height],
                uv: [0., 1.],
                ..vertex::Vertex2D::default()
            },
            vertex::Vertex2D {
                position: [x + width, y + height],
                uv: [1., 1.],
                ..vertex::Vertex2D::default()
            },
            vertex::Vertex2D {
                position: [x + width, y],
                uv: [1., 0.],
                ..vertex::Vertex2D::default()
            },
            vertex::Vertex2D {
                position: [x, y],
                uv: [0., 0.],
                ..vertex::Vertex2D::default()
            },
        ];
        self.polygon(DrawMode::Fill, &vertices, true)
    }

    pub fn polygon(
        &mut self,
        draw_mode: DrawMode,
        vertices: &[vertex::Vertex2D],
        skip_last_filled: bool,
    ) {
        match draw_mode {
            DrawMode::Fill => {
                let vertices = if skip_last_filled {
                    &vertices[0..vertices.len() - 1]
                } else {
                    &vertices
                };
                self.mesh.set_vertices(vertices, 0);
                self.mesh.set_draw_mode(graphics::DrawMode::TriangleFan);
                self.mesh.set_draw_range(Some(0..vertices.len()));
                self.mesh.draw(&mut self.gfx.borrow_mut());
            }
            DrawMode::Stroke => {
                use lyon_tessellation::*;
                let mut builder = path::Builder::new();
                builder.polygon(
                    &vertices
                        .iter()
                        .map(|v| v.position.into())
                        .collect::<Vec<_>>(),
                );
                let path = builder.build();

                let mut buffers: VertexBuffers<math::Point, u16> = VertexBuffers::new();
                {
                    let mut vertex_builder = geometry_builder::simple_builder(&mut buffers);
                    let mut tessellator = StrokeTessellator::new();
                    let _r = tessellator.tessellate(
                        &path,
                        &StrokeOptions::default(),
                        &mut vertex_builder,
                    );
                }
                let vertices = buffers
                    .indices
                    .iter()
                    .map(|i| {
                        let i = *i as usize;
                        vertex::Vertex2D {
                            position: [buffers.vertices[i].x, buffers.vertices[i].y],
                            ..vertex::Vertex2D::default()
                        }
                    })
                    .collect::<Vec<_>>();
                self.mesh.set_vertices(&vertices, 0);
                self.mesh.set_draw_mode(graphics::DrawMode::Triangles);
                self.mesh.set_draw_range(Some(0..vertices.len()));
                self.mesh.draw(&mut self.gfx.borrow_mut());
            }
        }
    }
}
