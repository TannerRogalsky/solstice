mod boilerplate;
use boilerplate::*;
use std::time::Duration;

struct Main {
    image: solstice::image::Image,
    deja_vu_sans: solstice_2d::FontId,
    pixel_font: solstice_2d::FontId,
    custom_shader: solstice_2d::Shader2D,
    tiling_noise: solstice::image::Image,
    gen_noise: solstice::image::Image,
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
            )
            .unwrap()
        };

        let tiling_noise = {
            let image = image::open(resources.join("tiling.png"))?;
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

        let gen_noise = {
            let mut rng = rand::thread_rng();
            solstice_2d::create_perlin_texture(
                &mut ctx.ctx,
                solstice_2d::PerlinTextureSettings {
                    rng: &mut rng,
                    width: 256,
                    height: 256,
                    period: 32,
                    levels: 1,
                    attenuation: 0.4,
                    absolute: false,
                    color: false,
                    alpha: false,
                },
            )?
        };

        let deja_vu_sans_font = {
            let path = resources.join("DejaVuSans.ttf");
            let font_data = std::fs::read(path).unwrap();
            let font = glyph_brush::ab_glyph::FontVec::try_from_vec(font_data).unwrap();
            ctx.ctx2d.add_font(font)
        };

        let pixel_font = {
            let path = resources.join("04b03.TTF");
            let font_data = std::fs::read(path).unwrap();
            let font = glyph_brush::ab_glyph::FontVec::try_from_vec(font_data).unwrap();
            ctx.ctx2d.add_font(font)
        };

        let custom_shader = {
            let path = resources.join("custom.glsl");
            let shader_src = std::fs::read_to_string(path).unwrap();
            let shader = solstice_2d::Shader2D::with(shader_src.as_str(), &mut ctx.ctx).unwrap();
            shader
        };

        Ok(Self {
            image,
            deja_vu_sans: deja_vu_sans_font,
            pixel_font,
            custom_shader,
            tiling_noise,
            gen_noise,
        })
    }

    fn draw(&mut self, ctx: &mut ExampleContext, time: Duration) {
        use solstice_2d::*;
        let mut ctx = ctx.ctx2d.start(&mut ctx.ctx);
        ctx.clear([0., 0., 0., 1.]);

        let image = &self.image;
        let custom_shader = &mut self.custom_shader;
        let deja_vu_sans = self.deja_vu_sans;
        let pixel_font = self.pixel_font;

        let t = time.as_secs_f32() % 10. / 10.;

        let circle = Circle {
            x: 200.,
            y: 200.0,
            radius: 100.0,
            segments: 6,
        };
        ctx.set_color([1., 0., 0., 1.]);
        ctx.draw(DrawMode::Fill, circle);
        ctx.set_color([0., 1., 0., 1.]);
        ctx.draw(DrawMode::Stroke, circle);

        ctx.set_color([1., 1., 1., 1.]);
        ctx.draw(DrawMode::Fill, &[(200., 0.), (100., 100.), (0., 100.)]);
        ctx.set_color([1., 0., 1., 1.]);
        ctx.draw(DrawMode::Stroke, [(200., 0.), (100., 100.), (0., 100.)]);

        let circle = Circle {
            x: 500.,
            y: 300.0,
            radius: 100.0,
            segments: 80,
        };
        ctx.set_color([1., 1., 1., 1.]);
        ctx.draw(DrawMode::Fill, circle);

        let image_rect = Rectangle {
            x: 100.,
            y: 300.,
            width: 512.,
            height: 512.,
        };
        ctx.image(image_rect, image);
        ctx.image(
            Rectangle {
                x: image_rect.x + 128.,
                y: image_rect.y + 128.,
                width: 256.,
                height: 256.,
            },
            image,
        );

        ctx.transforms.push();
        ctx.set_shader(custom_shader);
        let rectangle = Rectangle {
            x: 400. + (t * std::f32::consts::PI * 2.).sin() * 300.,
            y: 400. + (t * std::f32::consts::PI * 2.).cos() * 100.,
            width: 100.,
            height: 100.,
        };
        ctx.draw(DrawMode::Fill, rectangle);
        ctx.set_color([1., 0., 0., 1.]);
        ctx.draw(DrawMode::Stroke, rectangle);
        ctx.remove_active_shader();
        ctx.transforms.pop();

        ctx.transforms.push();
        let arc = Arc {
            arc_type: ArcType::Pie,
            x: 800.,
            y: 100.,
            radius: 25.0,
            angle1: Rad(0.),
            angle2: Rad(std::f32::consts::PI * 1.75),
            segments: 100,
        };
        ctx.set_color([1., 1., 1., 1.]);
        ctx.draw(DrawMode::Fill, arc);
        ctx.set_color([1., 0., 1., 1.]);
        ctx.draw(DrawMode::Stroke, arc);

        let arc = Arc {
            arc_type: ArcType::Closed,
            y: arc.y + 200.,
            ..arc
        };
        ctx.set_color([1., 1., 1., 1.]);
        ctx.draw(DrawMode::Fill, arc);
        ctx.set_color([1., 0., 1., 1.]);
        ctx.draw(DrawMode::Stroke, arc);

        let arc = Arc {
            arc_type: ArcType::Open,
            y: arc.y + 200.,
            ..arc
        };
        ctx.set_color([1., 1., 1., 1.]);
        ctx.draw(DrawMode::Fill, arc);
        ctx.set_color([1., 0., 1., 1.]);
        ctx.draw(DrawMode::Stroke, arc);

        ctx.transforms.pop();

        ctx.set_color([1., 1., 1., 1.]);
        ctx.print(deja_vu_sans, "Hello, World!", 0., 128., 128.);
        ctx.set_color([0.5, 0.1, 1., 0.25]);
        ctx.print(pixel_font, "Test", 128., 128., 256.);

        ctx.set_color([1., 1., 1., 1.]);
        ctx.line(0., 0., 400., 400.);
        ctx.line(400., 400., 0., 400.);
        ctx.line(0., 400., 0., 0.);

        {
            let t = ctx.transforms.push();
            t.translation_x = 10.;
            ctx.set_color([0.5, 0.1, 0.75, 0.5]);
            ctx.lines(&[(0., 0.), (400., 400.), (0., 400.), (0., 0.)]);
            ctx.transforms.pop();
        }

        let radius = 50.;
        for i in 3..12 {
            let x = (i - 2) as f32 * radius * 2.5;
            let y = 400.;
            let p = SimpleConvexPolygon {
                x: 0.,
                y: 0.,
                vertex_count: i as _,
                radius_x: radius,
                radius_y: radius,
            };
            let tx = ctx.transforms.push();
            *tx *= Transform::translation(x, y);
            *tx *= Transform::rotation(Rad(t * std::f32::consts::PI * 2.));
            ctx.set_color([1., 1., 1., 1.]);
            ctx.draw(DrawMode::Fill, p);
            ctx.set_color([0.1, 0.3, 0.9, 0.7]);
            ctx.draw(DrawMode::Stroke, p);
            ctx.line(0., 0., radius, 0.);
            ctx.transforms.pop();
        }

        let rectangle = Rectangle {
            x: 600.,
            y: 400.,
            width: 100.,
            height: 100.,
        };
        ctx.set_color([1., 1., 1., 1.]);
        for y in 0..3 {
            for x in 0..3 {
                let tx = ctx.transforms.push();
                *tx *=
                    Transform::translation(rectangle.width * x as f32, rectangle.height * y as f32);
                ctx.image(rectangle, &self.tiling_noise);
                ctx.transforms.pop();
            }
        }
        let rectangle = Rectangle {
            x: 900.,
            y: 400.,
            width: 100.,
            height: 100.,
        };
        for y in 0..3 {
            for x in 0..3 {
                let tx = ctx.transforms.push();
                *tx *=
                    Transform::translation(rectangle.width * x as f32, rectangle.height * y as f32);
                ctx.image(rectangle, &self.gen_noise);
                ctx.transforms.pop();
            }
        }
    }
}
fn main() {
    Main::run();
}
