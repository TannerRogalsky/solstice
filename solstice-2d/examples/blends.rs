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
            let mut dl = solstice_2d::DrawList::default();
            dl.set_canvas(Some(canvas.clone()));
            dl.clear([1., 0., 0., 1.]);

            ctx.gfx.process(&mut ctx.ctx, &mut dl);
        }
        Ok(Self { canvas })
    }

    fn draw(&mut self, ctx: &mut ExampleContext, _time: Duration) {
        use solstice_2d::*;
        let (width, height) = ctx.dimensions();
        let mut d2 = DrawList::default();
        d2.clear([0., 0., 0., 1.]);

        let rectangle = Rectangle {
            x: -50.,
            y: -50.,
            width: 100.,
            height: 100.,
        };

        let mut draw =
            |transform, color| d2.draw_with_color_and_transform(rectangle, color, transform);

        let origin = Transform2D::translation(width / 2., height / 2.);
        draw(origin, [1., 1., 1., 1.]);
        draw(
            origin * Transform2D::translation(50., 50.),
            [1., 0., 0., 0.5],
        );
        draw(
            origin * Transform2D::translation(50., -50.),
            [1., 0., 0., 1.],
        );
        d2.image_with_color_and_transform(
            rectangle,
            &self.canvas,
            [1., 1., 1., 0.5],
            origin * Transform2D::translation(-50., 50.),
        );
        d2.image_with_color_and_transform(
            rectangle,
            &self.canvas,
            [1., 1., 1., 1.],
            origin * Transform2D::translation(-50., -50.),
        );
        ctx.gfx.process(&mut ctx.ctx, &mut d2);
    }
}

fn main() {
    BlendExample::run();
}
