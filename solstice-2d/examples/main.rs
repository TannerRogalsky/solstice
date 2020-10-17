struct Resources<'a> {
    image: &'a solstice::image::Image,
    text: &'a mut solstice_2d::Text,
}

fn draw(mut ctx: solstice_2d::Graphics2DLock, resources: Resources) {
    use solstice_2d::*;
    let Resources { image, text } = resources;

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
    let rectangle = Rectangle {
        x: 400.,
        y: 400.,
        width: 100.,
        height: 100.,
    };
    ctx.rectangle(DrawMode::Fill, rectangle);
    ctx.set_color([1., 0., 0., 1.]);
    ctx.rectangle(DrawMode::Stroke, rectangle);
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

    text.draw(&mut ctx);
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

    let resources = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");

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

    let mut text = {
        let path = resources.join("DejaVuSans.ttf");
        let font_data = std::fs::read(path).unwrap();
        let font = glyph_brush::ab_glyph::FontArc::try_from_vec(font_data).unwrap();

        let mut text = solstice_2d::Text::new(&mut context, font).unwrap();
        text.set_text("Hello, World!", &mut d2.start(&mut context));
        text
    };

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
                        text: &mut text,
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
