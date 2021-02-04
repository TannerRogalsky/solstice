mod boilerplate;
use boilerplate::*;
use std::time::Duration;

struct FramebufferExample {
    bg_canvas: solstice_2d::Canvas,
    texture_canvas: solstice_2d::Canvas,
}

impl Example for FramebufferExample {
    fn new(ctx: &mut ExampleContext) -> eyre::Result<Self> {
        let (width, height) = ctx.dimensions();
        let bg_canvas = solstice_2d::Canvas::with_settings(&mut ctx.ctx, solstice_2d::solstice::canvas::Settings {
            width: width as _,
            height: height as _,
            with_depth: true,
            ..Default::default()
        })?;
        let texture_canvas = solstice_2d::Canvas::new(&mut ctx.ctx, width, height)?;

        {
            use solstice_2d::{Color, Draw, Rectangle};
            let mut dl = solstice_2d::DrawList::default();
            dl.set_canvas(Some(texture_canvas.clone()));
            dl.clear([1., 1., 1., 1.]);

            dl.draw_with_color(
                Rectangle {
                    x: 50.,
                    y: 50.,
                    width: width - 50. * 2.,
                    height: height - 50. * 2.,
                },
                Color::new(1., 0.3, 1., 1.),
            );

            ctx.gfx.process(&mut ctx.ctx, &mut dl);
        }

        Ok(Self {
            bg_canvas,
            texture_canvas,
        })
    }

    fn draw(&mut self, ctx: &mut ExampleContext, time: Duration) {
        use solstice_2d::*;
        let (width, height) = ctx.dimensions();
        let mut d2 = DrawList::default();
        d2.clear([0., 0., 0., 1.]);

        d2.set_canvas(Some(self.bg_canvas.clone()));
        d2.clear([0., 0., 0., 1.]);
        let tx = Transform3D::translation(0., 0., -2.);
        let tx = tx * Transform3D::rotation(Rad(0.), Rad(time.as_secs_f32()), Rad(0.));
        let cube = Box::new(1., 1., 1., 1, 1, 1);
        d2.image_with_transform(cube, &self.texture_canvas, tx);
        d2.set_canvas(None);

        let screen = Rectangle {
            x: 50.,
            y: 50.,
            width: width - 50. * 2.,
            height: height - 50. * 2.,
        };
        d2.image(screen, &self.bg_canvas);

        ctx.gfx.process(&mut ctx.ctx, &mut d2);
    }
}

fn main() {
    FramebufferExample::run();
}
