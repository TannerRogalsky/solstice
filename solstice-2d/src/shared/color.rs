#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl Color {
    pub const fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color {
            red: 1.,
            green: 1.,
            blue: 1.,
            alpha: 1.,
        }
    }
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> Self {
        [c.red, c.green, c.blue, c.alpha]
    }
}

impl From<[f32; 4]> for Color {
    fn from([red, green, blue, alpha]: [f32; 4]) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

impl From<Color> for solstice::Color<solstice::ClampedF32> {
    fn from(c: Color) -> Self {
        Self {
            red: c.red.into(),
            blue: c.blue.into(),
            green: c.green.into(),
            alpha: c.alpha.into(),
        }
    }
}

impl From<Color> for mint::Vector4<f32> {
    fn from(c: Color) -> Self {
        Self {
            x: c.red,
            y: c.blue,
            z: c.green,
            w: c.alpha,
        }
    }
}
