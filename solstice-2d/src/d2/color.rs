#[derive(Copy, Clone, Debug)]
pub struct Color {
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
}

impl Color {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
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

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        [self.red, self.green, self.blue, self.alpha]
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

impl Into<solstice::Color<solstice::ClampedF32>> for Color {
    fn into(self) -> solstice::Color<solstice::ClampedF32> {
        solstice::Color {
            red: self.red.into(),
            blue: self.blue.into(),
            green: self.green.into(),
            alpha: self.alpha.into(),
        }
    }
}
