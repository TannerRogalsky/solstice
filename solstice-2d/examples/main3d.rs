mod boilerplate;
use boilerplate::*;
use std::time::Duration;

const CUSTOM_SHADER: &str = r#"
varying vec3 cubeUV;

#ifdef VERTEX
vec4 pos(mat4 transform_projection, vec4 vertex_position) {
    cubeUV = vertex_position.xyz;
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
            let (width, height) = self.canvas.dimensions();
            let tx = solstice_2d::Transform2D::translation(width / 2.0, height / 2.0);
            // let tx = tx * solstice_2d::Transform::rotation(Rad(-t * std::f32::consts::PI * 2.0));
            dl.set_canvas(Some(self.canvas.clone()));
            dl.clear([1.0, 1.0, 1.0, 1.0]);
            dl.draw_with_color_and_transform(
                solstice_2d::RegularPolygon {
                    x: 0.0,
                    y: 0.0,
                    vertex_count: 8,
                    radius: height / 2.0,
                },
                [0.2, 0.4, 0.8, 1.0],
                tx,
            );
            dl.set_canvas(None);
        }

        let radius = 1.0;
        let color = [0.2, 0.4, 0.8, 1.0];
        let rotation = Transform3D::rotation(
            Rad(0.),
            Rad(t * std::f32::consts::PI * 2.),
            Rad(t * std::f32::consts::PI * 2.),
        );

        let polyhedra = vec![
            Polyhedron::tetrahedron(radius, 0),
            Polyhedron::octahedron(radius, 0),
            Polyhedron::icosahedron(radius, 0),
            Polyhedron::dodecahedron(radius, 0),
        ];

        dl.set_shader(Some(self.shader.clone()));
        let count = polyhedra.len() - 1;
        for (index, polyhedron) in polyhedra.into_iter().enumerate() {
            let x = ((index as f32 / count as f32) - 0.5) * 2.0 * radius * 4.0;
            let tx = Transform3D::translation(x, 0.0, -4.0) * rotation;
            dl.image_with_color_and_transform(polyhedron, &self.canvas, color, tx);
        }
        dl.set_shader(None);

        ctx.gfx.process(&mut ctx.ctx, &mut dl);
    }
}
fn main() {
    Main::run();
}
