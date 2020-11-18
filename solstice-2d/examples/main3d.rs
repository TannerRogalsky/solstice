mod boilerplate;
use boilerplate::*;
use solstice_2d::Rad;
use std::time::Duration;

struct Main {
    canvas: solstice_2d::Canvas,
}

impl Example for Main {
    fn new(ctx: &mut ExampleContext) -> eyre::Result<Self> {
        let canvas = solstice_2d::Canvas::new(&mut ctx.ctx, 720.0, 720.0)?;
        Ok(Self { canvas })
    }

    fn draw(&mut self, ctx: &mut ExampleContext, time: Duration) {
        use solstice_2d::d3::*;
        let t = time.as_secs_f32() % 3. / 3.;
        let (width, height) = ctx.dimensions();

        let mut dl = DrawList::default();
        dl.clear([1.0, 0.0, 0.0, 1.0]);

        let tx = solstice_2d::Transform::translation(width / 2.0, height / 2.0);
        dl.draw_with_transform(
            solstice_2d::RegularPolygon {
                x: 0.0,
                y: 0.0,
                vertex_count: 6,
                radius: height / 2.0,
            },
            tx,
        );

        {
            let (width, height) = self.canvas.dimensions();
            let tx = solstice_2d::Transform::translation(width / 2.0, height / 2.0);
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
        let rotation = Transform::rotation(
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

        let count = polyhedra.len() - 1;
        for (index, polyhedron) in polyhedra.into_iter().enumerate() {
            let x = ((index as f32 / count as f32) - 0.5) * 2.0 * radius * 4.0;
            let tx = Transform::translation(x, 0.0, -4.0) * rotation;
            dl.draw_with_color_and_transform(polyhedron, color, tx);
        }

        ctx.ctx3d.process(&mut ctx.ctx, &mut dl);
    }
}
fn main() {
    Main::run();
}
