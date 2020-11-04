pub struct ExampleContext {
    pub window: glutin::WindowedContext<glutin::PossiblyCurrent>,
    pub ctx: solstice::Context,
    pub ctx2d: solstice_2d::Graphics2D,
}

impl ExampleContext {
    pub fn dimensions(&self) -> (f32, f32) {
        let glutin::dpi::PhysicalSize { width, height } = self.window.window().inner_size();
        (width as f32, height as f32)
    }
}

pub trait Example: Sized {
    fn new(ctx: &mut ExampleContext) -> eyre::Result<Self>;
    fn draw(&mut self, ctx: &mut ExampleContext, time: std::time::Duration);
    fn run() -> !
    where
        Self: 'static,
    {
        use glutin::{
            event::*,
            event_loop::{ControlFlow, EventLoop},
            window::WindowBuilder,
        };

        let event_loop = EventLoop::new();
        let (width, height) = (1280, 720);
        let wb =
            WindowBuilder::new().with_inner_size(glutin::dpi::PhysicalSize::new(width, height));
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
        let d2 = solstice_2d::Graphics2D::new(&mut context, width as _, height as _).unwrap();

        let mut ctx = ExampleContext {
            window,
            ctx: context,
            ctx2d: d2,
        };
        let mut example = Self::new(&mut ctx).unwrap();

        let start = std::time::Instant::now();

        event_loop.run(move |event, _, cf| match event {
            Event::WindowEvent { window_id, event } => {
                if window_id == ctx.window.window().id() {
                    match event {
                        WindowEvent::CloseRequested => *cf = ControlFlow::Exit,
                        WindowEvent::Resized(glutin::dpi::PhysicalSize { width, height }) => {
                            ctx.ctx.set_viewport(0, 0, width as _, height as _);
                            ctx.ctx2d.set_width_height(width as _, height as _);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            let glutin::dpi::PhysicalSize { width, height } = *new_inner_size;
                            ctx.ctx.set_viewport(0, 0, width as _, height as _);
                            ctx.ctx2d.set_width_height(width as _, height as _);
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
                if window_id == ctx.window.window().id() {
                    example.draw(&mut ctx, start.elapsed());
                    ctx.window.swap_buffers().unwrap();
                }
            }
            Event::MainEventsCleared => {
                ctx.window.window().request_redraw();
            }
            _ => {}
        })
    }
}

#[allow(unused)]
fn main() {
    eprintln!("This isn't a real example.")
}
