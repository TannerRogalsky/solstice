fn draw(mut ctx: solstice_2d::Graphics2DLock) {
    use solstice_2d::*;

    ctx.circle(
        DrawMode::Fill,
        Circle {
            x: 200.,
            y: 200.0,
            radius: 100.0,
            segments: 100,
        },
    );

    ctx.rectangle(
        DrawMode::Fill,
        Rectangle {
            x: 400.,
            y: 200.,
            width: 100.,
            height: 100.,
        },
    );
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
                draw(d2.start(&mut context));
                window.swap_buffers().unwrap();
            }
        }
        Event::MainEventsCleared => {
            window.window().request_redraw();
        }
        _ => {}
    })
}
