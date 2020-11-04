mod boilerplate;
use boilerplate::*;
use std::time::Duration;

struct Big;

impl Example for Big {
    fn new(_ctx: &mut ExampleContext) -> eyre::Result<Self> {
        Ok(Self)
    }

    fn draw(&mut self, ctx: &mut ExampleContext, time: Duration) {
        let (width, height) = ctx.dimensions();
        use solstice_2d::*;

        let mut ctx = ctx.ctx2d.start(&mut ctx.ctx);
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
}

fn main() {
    Big::run();
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
