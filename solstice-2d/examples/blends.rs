mod boilerplate;
use boilerplate::*;
use std::time::Duration;

struct BlendExample {
    canvas: solstice_2d::Canvas,
}

impl Example for BlendExample {
    fn new(ctx: &mut ExampleContext) -> eyre::Result<Self> {
        let (width, height) = ctx.dimensions();
        let canvas = solstice_2d::Canvas::new(&mut ctx.ctx, width, height)?;
        {
            let mut d2 = ctx.ctx2d.start(&mut ctx.ctx);
            d2.set_canvas(&canvas);
            d2.clear([1., 0., 0., 1.]);
            d2.unset_canvas();
        }
        Ok(Self { canvas })
    }

    fn draw(&mut self, ctx: &mut ExampleContext, _time: Duration) {
        use solstice_2d::*;
        let (width, height) = ctx.dimensions();
        let mut d2 = ctx.ctx2d.start(&mut ctx.ctx);
        d2.clear([0., 0., 0., 1.]);

        let rectangle = Rectangle {
            x: -50.,
            y: -50.,
            width: 100.,
            height: 100.,
        };

        let mut draw = |transform, color| {
            d2.draw_with_transform_and_color(DrawMode::Fill, rectangle, transform, color)
        };

        let origin = Transform::translation(width / 2., height / 2.);
        draw(origin, [1., 1., 1., 1.]);
        draw(origin * Transform::translation(50., 50.), [1., 0., 0., 0.5]);
        draw(origin * Transform::translation(50., -50.), [1., 0., 0., 1.]);
        d2.image_with_transform_and_color(
            rectangle,
            &self.canvas,
            origin * Transform::translation(-50., 50.),
            [1., 1., 1., 0.5],
        );
        d2.image_with_transform_and_color(
            rectangle,
            &self.canvas,
            origin * Transform::translation(-50., -50.),
            [1., 1., 1., 1.],
        );
    }
}

fn main() {
    BlendExample::run();
}
