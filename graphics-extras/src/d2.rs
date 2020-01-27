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

pub enum ArcType {
    Pie,
    Open,
    Closed,
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

    pub fn arc(
        &mut self,
        draw_mode: DrawMode,
        x: f32,
        y: f32,
        radius: f32,
        angle1: f32,
        angle2: f32,
    ) {
        self.arc_with_type(draw_mode, ArcType::Pie, x, y, radius, angle1, angle2)
    }
    pub fn arc_with_segments(
        &mut self,
        draw_mode: DrawMode,
        x: f32,
        y: f32,
        radius: f32,
        angle1: f32,
        angle2: f32,
        segments: u32,
    ) {
        self.arc_with_type_and_segments(
            draw_mode,
            ArcType::Pie,
            x,
            y,
            radius,
            angle1,
            angle2,
            segments,
        )
    }
    pub fn arc_with_type(
        &mut self,
        draw_mode: DrawMode,
        arc_type: ArcType,
        x: f32,
        y: f32,
        radius: f32,
        angle1: f32,
        angle2: f32,
    ) {
        self.arc_with_type_and_segments(
            draw_mode,
            arc_type,
            x,
            y,
            radius,
            angle1,
            angle2,
            radius.max(8.) as u32,
        )
    }
    pub fn arc_with_type_and_segments(
        &mut self,
        draw_mode: DrawMode,
        arc_type: ArcType,
        x: f32,
        y: f32,
        radius: f32,
        angle1: f32,
        angle2: f32,
        segments: u32,
    ) {
        if segments == 0 || angle1 == angle2 {
            return;
        }

        const TWO_PI: f32 = std::f32::consts::PI * 2.;
        if (angle1 - angle2).abs() >= TWO_PI {
            return self.circle_with_segments(draw_mode, x, y, radius, segments);
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
        self.mesh.set_draw_mode(graphics::DrawMode::TriangleFan);
        self.mesh.set_vertices(&coords, 0);
        self.mesh.set_draw_range(Some(0..coords.len()));
        self.mesh.draw(&mut self.gfx.borrow_mut());
    }
    pub fn circle(&mut self, draw_mode: DrawMode, x: f32, y: f32, radius: f32) {
        self.ellipse(draw_mode, x, y, radius, radius);
    }
    pub fn circle_with_segments(
        &mut self,
        draw_mode: DrawMode,
        x: f32,
        y: f32,
        radius: f32,
        segments: u32,
    ) {
        self.ellipse_with_segments(draw_mode, x, y, radius, radius, segments);
    }
    pub fn ellipse(&mut self, draw_mode: DrawMode, x: f32, y: f32, radius_x: f32, radius_y: f32) {
        self.ellipse_with_segments(
            draw_mode,
            x,
            y,
            radius_x,
            radius_y,
            radius_x.max(radius_y).max(8.) as u32,
        );
    }
    pub fn ellipse_with_segments(
        &mut self,
        draw_mode: DrawMode,
        x: f32,
        y: f32,
        radius_x: f32,
        radius_y: f32,
        segments: u32,
    ) {
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
        self.mesh.set_vertices(&vertices, 0);
        self.mesh.set_draw_range(Some(0..vertices.len()));
        self.mesh.set_draw_mode(graphics::DrawMode::TriangleFan);
        self.mesh.draw(&mut self.gfx.borrow_mut());
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
    pub fn rectangle(&mut self, draw_mode: DrawMode, x: f32, y: f32, width: f32, height: f32) {
        self.default_shader.bind_texture(&self.default_texture);
        let quad = graphics::quad_batch::Quad::from(graphics::viewport::Viewport::new(
            x, y, width, height,
        ))
        .map(|(x, y)| vertex::Vertex2D::new([x, y], [1., 1., 1., 1.], [0.5, 0.5]));
        self.mesh.set_vertices(&quad.vertices, 0);
        self.mesh.set_draw_range(Some(0..4));
        self.mesh.set_draw_mode(graphics::DrawMode::TriangleStrip);
        self.mesh.draw(&mut self.gfx.borrow_mut());
    }
    pub fn image<T>(&mut self, x: f32, y: f32, width: f32, height: f32, texture: T)
    where
        T: graphics::texture::Texture,
    {
        use graphics::{quad_batch::Quad, viewport::Viewport};
        self.default_shader.bind_texture(texture);
        let quad = Quad::from(Viewport::new(x, y, width, height))
            .zip(Quad::from(Viewport::new(0., 0., 1., 1.)))
            .map(|((x, y), (s, t))| vertex::Vertex2D::new([x, y], [1., 1., 1., 1.], [s, t]));
        self.mesh.set_vertices(&quad.vertices, 0);
        self.mesh.set_draw_range(Some(0..4));
        self.mesh.set_draw_mode(graphics::DrawMode::TriangleStrip);
        self.mesh.draw(&mut self.gfx.borrow_mut());
    }
}
