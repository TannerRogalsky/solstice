use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Zeroable, Pod, solstice::vertex::Vertex, Copy, Clone, Debug, PartialEq)]
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

impl From<(f32, f32)> for Vertex2D {
    fn from((x, y): (f32, f32)) -> Self {
        Self {
            position: [x, y],
            ..Default::default()
        }
    }
}

impl From<(f64, f64)> for Vertex2D {
    fn from((x, y): (f64, f64)) -> Self {
        Self {
            position: [x as _, y as _],
            ..Default::default()
        }
    }
}

impl From<Point> for Vertex2D {
    fn from(p: Point) -> Self {
        Self {
            position: [p.x, p.y],
            ..Default::default()
        }
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

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
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

impl<T> From<&T> for Point
where
    Self: From<T>,
    T: Copy,
{
    fn from(p: &T) -> Self {
        Into::into(*p)
    }
}

impl From<Point> for mint::Point2<f32> {
    fn from(p: Point) -> Self {
        Self { x: p.x, y: p.y }
    }
}

impl From<mint::Point2<f32>> for Point {
    fn from(p: mint::Point2<f32>) -> Self {
        Self { x: p.x, y: p.y }
    }
}
