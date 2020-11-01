fn draw(mut ctx: solstice_2d::Graphics2DLock, width: f32, height: f32, time: std::time::Duration) {
    use solstice_2d::*;

    let cycle = 3.;
    let t = time.as_secs_f32() % cycle / cycle;
    let circle = Circle {
        x: 0.0,
        y: 0.0,
        radius: 10.0,
        segments: 25,
    };

    let half_width = width / 2.;
    let half_height = height / 2.;
    let ox = half_width;
    let oy = half_height;

    const TOTAL: u32 = 5000;
    const TAU: f32 = 6.28318530717958647692528676655900577_f32;
    for i in 0..TOTAL {
        let ratio = i as f32 / TOTAL as f32;
        let phi = (ratio * TAU + t) % TAU * 5.;

        let b = 0.035;
        let c = 1.;
        let radius = b * phi.powf(1. / c);

        let dx = phi.cos() * half_width * radius;
        let dy = phi.sin() * half_height * radius;

        let color = to_rgba(ratio * TAU + t, 1.0, 0.5);
        ctx.set_color(color);
        ctx.draw(
            DrawMode::Fill,
            Circle {
                x: ox + dx,
                y: oy + dy,
                ..circle
            },
        );
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
                let (width, height) = d2.dimensions();
                draw(d2.start(&mut context), width, height, start.elapsed());
                window.swap_buffers().unwrap();
            }
        }
        Event::MainEventsCleared => {
            window.window().request_redraw();
        }
        _ => {}
    })
}

fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
    // Normalize
    let t = if t < 0.0 {
        t + 1.0
    } else if t > 1.0 {
        t - 1.0
    } else {
        t
    };

    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}

fn to_rgba(h: f32, s: f32, l: f32) -> [f32; 4] {
    if s == 0.0 {
        // Achromatic, i.e., grey.
        return [l, l, l, 1.];
    }

    let h = h / (std::f32::consts::PI * 2.);
    let s = s;
    let l = l;

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - (l * s)
    };
    let p = 2.0 * l - q;

    [
        hue_to_rgb(p, q, h + 1.0 / 3.0),
        hue_to_rgb(p, q, h),
        hue_to_rgb(p, q, h - 1.0 / 3.0),
        1.,
    ]
}
