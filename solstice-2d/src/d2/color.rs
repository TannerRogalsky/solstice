#[derive(Copy, Clone, Debug)]
pub struct Color {
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
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
