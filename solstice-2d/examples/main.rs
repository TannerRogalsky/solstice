struct Resources<'a> {
    image: &'a solstice::image::Image,
    deja_vu_sans: solstice_2d::FontId,
    pixel_font: solstice_2d::FontId,
    custom_shader: &'a mut solstice_2d::Shader2D,
    time: std::time::Duration,
}

fn draw<'b, 'c: 'b>(mut ctx: solstice_2d::Graphics2DLock<'_, 'b>, resources: Resources<'c>) {
    use solstice_2d::*;
    let Resources {
        image,
        deja_vu_sans,
        pixel_font,
        custom_shader,
        time,
    } = resources;

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
}

fn main() {
    use glutin::{
        event::*,
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    };

    let event_loop = EventLoop::new();
    let (width, height) = (1280, 720);
    let wb = WindowBuilder::new().with_inner_size(glutin::dpi::PhysicalSize::new(width, height));
    let window = glutin::ContextBuilder::new()
        .with_multisampling(16)
        .with_double_buffer(Some(true))
        .with_vsync(true)
        .build_windowed(wb, &event_loop)
        .unwrap();
    let window = unsafe { window.make_current().unwrap() };
    let glow_ctx = unsafe {
        solstice::glow::Context::from_loader_function(|name| window.get_proc_address(name))
    };
    let mut context = solstice::Context::new(glow_ctx);
    let mut d2 = solstice_2d::Graphics2D::new(&mut context, width as _, height as _).unwrap();

    let resources = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("resources");

    let image = {
        let image = image::open(resources.join("rust-logo-512x512.png")).unwrap();
        let image = image.as_rgba8().unwrap();

        solstice::image::Image::with_data(
            &mut context,
            solstice::texture::TextureType::Tex2D,
            solstice::PixelFormat::RGBA8,
            image.width(),
            image.height(),
            image.as_raw(),
            solstice::image::Settings::default(),
        )
        .unwrap()
    };

    let deja_vu_sans_font = {
        let path = resources.join("DejaVuSans.ttf");
        let font_data = std::fs::read(path).unwrap();
        let font = glyph_brush::ab_glyph::FontVec::try_from_vec(font_data).unwrap();
        d2.add_font(font)
    };

    let pixel_font = {
        let path = resources.join("04b03.TTF");
        let font_data = std::fs::read(path).unwrap();
        let font = glyph_brush::ab_glyph::FontVec::try_from_vec(font_data).unwrap();
        d2.add_font(font)
    };

    let mut custom_shader = {
        let path = resources.join("custom.glsl");
        let shader_src = std::fs::read_to_string(path).unwrap();
        let shader =
            solstice_2d::Shader2D::with(shader_src.as_str(), &mut context, width as _, height as _)
                .unwrap();
        shader
    };

    let start = std::time::Instant::now();

    event_loop.run(move |event, _, cf| match event {
        Event::WindowEvent { window_id, event } => {
            if window_id == window.window().id() {
                match event {
                    WindowEvent::CloseRequested => *cf = ControlFlow::Exit,
                    WindowEvent::Resized(glutin::dpi::PhysicalSize { width, height }) => {
                        context.set_viewport(0, 0, width as _, height as _);
                        d2.set_width_height(width as _, height as _);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        let glutin::dpi::PhysicalSize { width, height } = new_inner_size;
                        d2.set_width_height(*width as _, *height as _);
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *cf = ControlFlow::Exit,
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id) => {
            if window_id == window.window().id() {
                context.clear();
                draw(
                    d2.start(&mut context),
                    Resources {
                        image: &image,
                        deja_vu_sans: deja_vu_sans_font,
                        pixel_font,
                        custom_shader: &mut custom_shader,
                        time: start.elapsed(),
                    },
                );
                window.swap_buffers().unwrap();
            }
        }
        Event::MainEventsCleared => {
            window.window().request_redraw();
        }
        _ => {}
    })
}
