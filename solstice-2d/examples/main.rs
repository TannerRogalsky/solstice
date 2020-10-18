struct Resources<'a> {
    image: &'a solstice::image::Image,
    deja_vu_sans: solstice_2d::FontId,
    pixel_font: solstice_2d::FontId,
    custom_shader: &'a mut solstice::shader::DynamicShader,
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
    ctx.circle(DrawMode::Fill, circle);

    ctx.set_color([0., 1., 0., 1.]);
    ctx.circle(DrawMode::Stroke, circle);

    let circle = Circle {
        x: 500.,
        y: 300.0,
        radius: 100.0,
        segments: 80,
    };
    ctx.set_color([1., 1., 1., 1.]);
    ctx.circle(DrawMode::Fill, circle);

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
    ctx.rectangle(DrawMode::Fill, rectangle);
    ctx.set_color([1., 0., 0., 1.]);
    ctx.rectangle(DrawMode::Stroke, rectangle);
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
    ctx.arc(DrawMode::Fill, arc);
    ctx.set_color([1., 0., 1., 1.]);
    ctx.arc(DrawMode::Stroke, arc);

    let arc = Arc {
        arc_type: ArcType::Closed,
        y: arc.y + 200.,
        ..arc
    };
    ctx.set_color([1., 1., 1., 1.]);
    ctx.arc(DrawMode::Fill, arc);
    ctx.set_color([1., 0., 1., 1.]);
    ctx.arc(DrawMode::Stroke, arc);

    let arc = Arc {
        arc_type: ArcType::Open,
        y: arc.y + 200.,
        ..arc
    };
    ctx.set_color([1., 1., 1., 1.]);
    ctx.arc(DrawMode::Fill, arc);
    ctx.set_color([1., 0., 1., 1.]);
    ctx.arc(DrawMode::Stroke, arc);

    ctx.transforms.pop();

    ctx.set_color([1., 1., 1., 1.]);
    ctx.print(deja_vu_sans, "Hello, World!", 0., 0., 128.);
    ctx.set_color([0.5, 0.1, 1., 1.]);
    ctx.print(pixel_font, "Test", 0., 128., 50.);
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
        let (vert, frag) = solstice::shader::DynamicShader::create_source(&shader_src, &shader_src);
        let shader = solstice::shader::DynamicShader::new(&mut context, &vert, &frag).unwrap();
        context.use_shader(Some(&shader));
        context.set_uniform_by_location(
            &shader.get_uniform_by_name("uProjection").unwrap().location,
            &solstice::shader::RawUniformValue::Mat4(ortho(width as _, height as _).into()),
        );
        shader
    };

    let start = std::time::Instant::now();

    event_loop.run(move |event, _, cf| match event {
        Event::WindowEvent { window_id, event } => {
            if window_id == window.window().id() {
                match event {
                    WindowEvent::CloseRequested => *cf = ControlFlow::Exit,
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

fn ortho(width: f32, height: f32) -> [[f32; 4]; 4] {
    let left = 0.;
    let right = width;
    let bottom = height;
    let top = 0.;
    let near = 0.;
    let far = 1000.;

    let c0r0 = 2. / (right - left);
    let c0r1 = 0.;
    let c0r2 = 0.;
    let c0r3 = 0.;

    let c1r0 = 0.;
    let c1r1 = 2. / (top - bottom);
    let c1r2 = 0.;
    let c1r3 = 0.;

    let c2r0 = 0.;
    let c2r1 = 0.;
    let c2r2 = -2. / (far - near);
    let c2r3 = 0.;

    let c3r0 = -(right + left) / (right - left);
    let c3r1 = -(top + bottom) / (top - bottom);
    let c3r2 = -(far + near) / (far - near);
    let c3r3 = 1.;

    #[cfg_attr(rustfmt, rustfmt_skip)]
    [
        [c0r0, c0r1, c0r2, c0r3],
        [c1r0, c1r1, c1r2, c1r3],
        [c2r0, c2r1, c2r2, c2r3],
        [c3r0, c3r1, c3r2, c3r3],
    ]
}
