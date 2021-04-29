mod boilerplate;
use boilerplate::*;
use std::time::Duration;

struct Main {
    image: solstice::image::Image,
    deja_vu_sans: solstice_2d::FontId,
    pixel_font: solstice_2d::FontId,
    custom_shader: solstice_2d::Shader,
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
            let image = image::open(resources.join("noise_seed1-cell16-level2-att4_32.png"))?;
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
            solstice_2d::create_perlin_texture(
                &mut ctx.ctx,
                solstice_2d::PerlinTextureSettings {
                    seed: 1,
                    width: 32,
                    height: 32,
                    period: 16,
                    levels: 2,
                    attenuation: std::convert::TryInto::try_into(0.4).unwrap(),
                    color: true,
                },
            )?
        };

        let deja_vu_sans_font = {
            let path = resources.join("DejaVuSans.ttf");
            let font_data = std::fs::read(path).unwrap();
            let font = std::convert::TryInto::try_into(font_data).unwrap();
            ctx.gfx.add_font(font)
        };

        let pixel_font = {
            let path = resources.join("04b03.TTF");
            let font_data = std::fs::read(path).unwrap();
            let font = std::convert::TryInto::try_into(font_data).unwrap();
            ctx.gfx.add_font(font)
        };

        let custom_shader = {
            let path = resources.join("custom.glsl");
            let shader_src = std::fs::read_to_string(path).unwrap();
            let shader = solstice_2d::Shader::with(shader_src.as_str(), &mut ctx.ctx).unwrap();
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
        let (width, height) = ctx.dimensions();
        let mut dl = DrawList::default();
        dl.clear([0., 0., 0., 1.]);

        let image = &self.image;
        let custom_shader = self.custom_shader.clone();
        let deja_vu_sans = self.deja_vu_sans;
        let pixel_font = self.pixel_font;

        let t = time.as_secs_f32() % 10. / 10.;

        let circle = Circle {
            x: 200.,
            y: 200.0,
            radius: 100.0,
            segments: 6,
        };
        dl.set_color([1., 0., 0., 1.]);
        dl.draw(circle);
        dl.set_color([0., 1., 0., 1.]);
        dl.stroke(circle);

        dl.set_color([1., 1., 1., 1.]);
        dl.draw(&[(200., 0.), (100., 100.), (0., 100.)]);
        dl.set_color([1., 0., 1., 1.]);
        dl.stroke([(200., 0.), (100., 100.), (0., 100.)]);

        let circle = Circle {
            x: 500.,
            y: 300.0,
            radius: 100.0,
            segments: 80,
        };
        dl.set_color([1., 1., 1., 1.]);
        dl.draw(circle);

        let image_rect = Rectangle {
            x: 100.,
            y: 300.,
            width: 512.,
            height: 512.,
        };
        dl.image(image_rect, image);
        dl.image(
            Rectangle {
                x: image_rect.x + 128.,
                y: image_rect.y + 128.,
                width: 256.,
                height: 256.,
            },
            image,
        );

        dl.set_shader(Some(custom_shader));
        let rectangle = Rectangle {
            x: 400. + (t * std::f32::consts::PI * 2.).sin() * 300.,
            y: 400. + (t * std::f32::consts::PI * 2.).cos() * 100.,
            width: 100.,
            height: 100.,
        };
        dl.draw(rectangle);
        dl.set_color([1., 0., 0., 1.]);
        dl.stroke(rectangle);
        dl.set_shader(None);

        let arc = Arc {
            arc_type: ArcType::Pie,
            x: 800.,
            y: 100.,
            radius: 25.0,
            angle1: Rad(0.),
            angle2: Rad(std::f32::consts::PI * 1.75),
            segments: 100,
        };
        dl.set_color([1., 1., 1., 1.]);
        dl.draw(arc);
        dl.set_color([1., 0., 1., 1.]);
        dl.stroke(arc);

        let arc = Arc {
            arc_type: ArcType::Closed,
            y: arc.y + 200.,
            ..arc
        };
        dl.set_color([1., 1., 1., 1.]);
        dl.draw(arc);
        dl.set_color([1., 0., 1., 1.]);
        dl.stroke(arc);

        let arc = Arc {
            arc_type: ArcType::Open,
            y: arc.y + 200.,
            ..arc
        };
        dl.set_color([1., 1., 1., 1.]);
        dl.draw(arc);
        dl.set_color([1., 0., 1., 1.]);
        dl.stroke(arc);

        dl.set_color([1., 1., 1., 1.]);
        dl.print(
            String::from("Hello, World!"),
            deja_vu_sans,
            64.,
            Rectangle {
                x: 128.,
                y: 128.,
                width,
                height,
            },
        );
        dl.set_color([0.5, 0.1, 1., 0.25]);
        dl.print(
            "Test",
            pixel_font,
            128.,
            Rectangle {
                x: 128.,
                y: 256.,
                width,
                height,
            },
        );

        {
            let t = Transform2D::translation(10., 0.);
            dl.set_transform(t);
            dl.set_color([0.5, 0.1, 0.75, 0.5]);
            dl.line_2d(
                [(0., 0.), (400., 400.), (0., 400.), (0., 0.)]
                    .iter()
                    .map(|(x, y)| LineVertex {
                        position: [*x, *y, 0.],
                        width: 10.,
                        ..LineVertex::default()
                    })
                    .collect::<Vec<_>>(),
            );
            dl.set_transform(Transform2D::default());
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
            let mut tx = Transform2D::default();
            tx *= Transform2D::translation(x, y);
            tx *= Transform2D::rotation(Rad(t * std::f32::consts::PI * 2.));
            dl.set_transform(tx);
            dl.set_color([1., 1., 1., 1.]);
            dl.draw(p);
            dl.set_color([0.1, 0.3, 0.9, 0.7]);
            dl.stroke(p);
            // dl.line(0., 0., radius, 0.);
            dl.set_transform(Transform2D::default());
        }

        let rectangle = Rectangle {
            x: 600.,
            y: 400.,
            width: 100.,
            height: 100.,
        };
        dl.set_color([1., 1., 1., 1.]);
        for y in 0..3 {
            for x in 0..3 {
                dl.image_with_transform(
                    rectangle,
                    &self.tiling_noise,
                    Transform2D::translation(
                        rectangle.width * x as f32,
                        rectangle.height * y as f32,
                    ),
                );
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
                let tx = Transform2D::translation(
                    rectangle.width * x as f32,
                    rectangle.height * y as f32,
                );
                dl.image_with_transform(rectangle, &self.gen_noise, tx);
            }
        }

        ctx.gfx.process(&mut ctx.ctx, &mut dl);
    }
}
fn main() {
    Main::run();
}
