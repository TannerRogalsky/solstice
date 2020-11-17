mod boilerplate;
use boilerplate::*;
use solstice_2d::Rad;
use std::time::Duration;

struct Main {
    image: solstice::image::Image,
}

impl Example for Main {
    fn new(ctx: &mut ExampleContext) -> eyre::Result<Self> {
        let resources = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("examples")
            .join("resources");

        let image = {
            let image = image::open(resources.join("rust-logo-512x512.png")).unwrap();
            let image = image.as_rgba8().unwrap();

            solstice::image::Image::with_data(
                &mut ctx.ctx,
                solstice::texture::TextureType::Tex2D,
                solstice::PixelFormat::RGBA8,
                image.width(),
                image.height(),
                image.as_raw(),
                solstice::image::Settings::default(),
            )?
        };

        Ok(Self { image })
    }

    fn draw(&mut self, ctx: &mut ExampleContext, time: Duration) {
        use solstice_2d::d3::*;
        let t = time.as_secs_f32() % 3. / 3.;

        let mut dl = DrawList::default();
        dl.clear([1.0, 0.0, 0.0, 1.0]);

        let tx = Transform::translation(0., 0., -2.0);
        let tx = tx
            * Transform::rotation(
                Rad(0.),
                Rad(t * std::f32::consts::PI * 2.),
                Rad(t * std::f32::consts::PI * 2.),
            );
        let box_geometry = Box::default();
        dl.image_with_transform(box_geometry, &self.image, tx);

        ctx.ctx3d.process(&mut ctx.ctx, &mut dl);
    }
}
fn main() {
    Main::run();
}
