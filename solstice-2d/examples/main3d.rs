mod boilerplate;
use boilerplate::*;
use std::time::Duration;

const CUSTOM_SHADER: &str = r#"
varying vec3 cubeUV;

#ifdef VERTEX
vec4 pos(mat4 transform_projection, vec4 vertex_position) {
    cubeUV = (vertex_position.xyz + 1.) / 2.;
    return transform_projection * vertex_position;
}
#endif

#ifdef FRAGMENT
vec4 effect(vec4 color, Image texture, vec2 st, vec2 screen_coords) {
    return vec4(cubeUV, 1.0);
}
#endif
"#;

struct Main {
    canvas: solstice_2d::Canvas,
    shader: solstice_2d::Shader,
}

impl Main {
    fn draw_polyhedra(&self, state: &solstice_2d::DrawList, t: f32) -> solstice_2d::DrawList {
        use solstice_2d::{Draw, Polyhedron};

        let radius = 1.0;
        let color = [0.2, 0.4, 0.8, 1.0];
        let rotation = solstice_2d::Transform3D::rotation(
            solstice_2d::Rad(0.),
            solstice_2d::Rad(t * std::f32::consts::PI * 2.),
            solstice_2d::Rad(t * std::f32::consts::PI * 2.),
        );

        let polyhedra = vec![
            Polyhedron::tetrahedron(radius, 0),
            Polyhedron::octahedron(radius, 0),
            Polyhedron::icosahedron(radius, 0),
            Polyhedron::dodecahedron(radius, 0),
        ];

        let mut dl = solstice_2d::DrawList::new_from_state(state);
        dl.set_shader(Some(self.shader.clone()));
        let count = polyhedra.len() - 1;
        for (index, polyhedron) in polyhedra.into_iter().enumerate() {
            let x = ((index as f32 / count as f32) - 0.5) * 2.0 * radius * 4.0;
            let tx = solstice_2d::Transform3D::translation(x, 0.0, -4.0) * rotation;
            dl.image_with_color_and_transform(polyhedron, &self.canvas, color, tx);
        }
        dl
    }
}

impl Example for Main {
    fn new(ctx: &mut ExampleContext) -> eyre::Result<Self> {
        let canvas = solstice_2d::Canvas::new(&mut ctx.ctx, 720.0, 720.0)?;
        let shader = solstice_2d::Shader::with(CUSTOM_SHADER, &mut ctx.ctx)?;
        Ok(Self { canvas, shader })
    }

    fn draw(&mut self, ctx: &mut ExampleContext, time: Duration) {
        use solstice_2d::*;
        let t = time.as_secs_f32() % 3. / 3.;
        let (width, height) = ctx.dimensions();

        let mut dl = DrawList::default();
        dl.clear([1.0, 0.0, 0.0, 1.0]);

        let tx = solstice_2d::Transform2D::translation(width / 2.0, height / 2.0);
        let bg_shape = solstice_2d::RegularPolygon {
            x: 0.0,
            y: 0.0,
            vertex_count: 6,
            radius: height / 2.0,
        };
        dl.draw_with_transform(bg_shape, tx);
        dl.stroke_with_color_and_transform(bg_shape, [0.0, 0.0, 0.0, 1.0], tx);

        {
            let mut dl2 = DrawList::new_from_state(&dl);
            let (width, height) = self.canvas.dimensions();
            let tx = solstice_2d::Transform2D::translation(width / 2.0, height / 2.0);
            // let tx = tx * solstice_2d::Transform::rotation(Rad(-t * std::f32::consts::PI * 2.0));
            dl2.set_canvas(Some(self.canvas.clone()));
            dl2.clear([1.0, 1.0, 1.0, 1.0]);
            dl2.draw_with_color_and_transform(
                solstice_2d::RegularPolygon {
                    x: 0.0,
                    y: 0.0,
                    vertex_count: 8,
                    radius: height / 2.0,
                },
                [0.2, 0.4, 0.8, 1.0],
                tx,
            );
            dl.append(&mut dl2);
        }

        {
            let mut dl2 = self.draw_polyhedra(&dl, t);
            dl.append(&mut dl2);
        }

        let offset = [0., 0.];
        let dim = [2., 2.];
        let point_count = 7u32;

        let point_generator = |t: f32| {
            (0..point_count).map(move |i| {
                let phi = i as f32 / (point_count - 1) as f32;
                let tau = std::f32::consts::PI * 2.;
                let [ox, oy] = offset;
                let [dx, dy] = dim;
                let (x, y) = ((phi + t) * tau).sin_cos();
                LineVertex {
                    position: [ox + x * dx, oy + y * dy, -8.0 + (t * tau).sin() * 5.0],
                    width: (1. - phi) * 10.0 + 5.0,
                    color: [0.4, phi, 0.2, 1.0],
                }
            })
        };

        let ring_count = 7;
        for ring in 0..ring_count {
            let phi = ring as f32 / ring_count as f32;
            dl.line_3d(point_generator(t + phi).collect::<Vec<_>>());
        }

        ctx.gfx.process(&mut ctx.ctx, &mut dl);
    }
}
fn main() {
    Main::run();
}
