use solstice::vertex::Vertex;

#[repr(C)]
#[derive(Vertex, Copy, Clone, Debug)]
pub struct Vertex2D {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

impl Default for Vertex2D {
    fn default() -> Self {
        Self {
            position: [0., 0.],
            color: [1., 1., 1., 1.],
            uv: [0.5, 0.5],
        }
    }
}

impl Vertex2D {
    pub fn new(position: [f32; 2], color: [f32; 4], uv: [f32; 2]) -> Self {
        Self {
            position,
            color,
            uv,
        }
    }

    pub fn position(&self) -> &[f32; 2] {
        &self.position
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<(f32, f32)> for Point {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

impl From<[f32; 2]> for Point {
    fn from([x, y]: [f32; 2]) -> Self {
        Self { x, y }
    }
}

impl From<lyon_tessellation::math::Point> for Point {
    fn from(p: lyon_tessellation::math::Point) -> Self {
        Self { x: p.x, y: p.y }
    }
}

impl Into<lyon_tessellation::math::Point> for Point {
    fn into(self) -> lyon_tessellation::math::Point {
        lyon_tessellation::math::Point::new(self.x, self.y)
    }
}

impl Into<lyon_tessellation::math::Point> for &Point {
    fn into(self) -> lyon_tessellation::math::Point {
        lyon_tessellation::math::Point::new(self.x, self.y)
    }
}

impl<T> From<&T> for Point
where
    Self: From<T>,
    T: Copy,
{
    fn from(p: &T) -> Self {
        Into::into(*p)
    }
}

impl Into<mint::Point2<f32>> for Point {
    fn into(self) -> mint::Point2<f32> {
        mint::Point2 {
            x: self.x,
            y: self.y,
        }
    }
}

impl From<mint::Point2<f32>> for Point {
    fn from(p: mint::Point2<f32>) -> Self {
        Self { x: p.x, y: p.y }
    }
}
