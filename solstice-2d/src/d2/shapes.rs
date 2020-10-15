/// An angle, in radians.
#[derive(Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Rad(pub f32);

/// An angle, in degrees.
#[derive(Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Deg(pub f32);

impl From<Rad> for Deg {
    #[inline]
    fn from(rad: Rad) -> Deg {
        Deg(rad.0 * 180.0 / std::f32::consts::PI)
    }
}

impl From<Deg> for Rad {
    #[inline]
    fn from(deg: Deg) -> Rad {
        Rad(deg.0 * std::f32::consts::PI / 180.0)
    }
}

#[derive(Copy, Clone, Default)]
pub struct Arc {
    pub arc_type: ArcType,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub angle1: Rad,
    pub angle2: Rad,
    pub segments: u32,
}

#[derive(Copy, Clone)]
pub enum ArcType {
    Pie,
    Open,
    Closed,
}

impl Default for ArcType {
    fn default() -> Self {
        ArcType::Pie
    }
}

#[derive(Copy, Clone, Default)]
pub struct Circle {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub segments: u32,
}

impl Into<Ellipse> for Circle {
    fn into(self) -> Ellipse {
        Ellipse {
            x: self.x,
            y: self.y,
            radius_x: self.radius,
            radius_y: self.radius,
            segments: self.segments,
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct Ellipse {
    pub x: f32,
    pub y: f32,
    pub radius_x: f32,
    pub radius_y: f32,
    pub segments: u32,
}

#[derive(Copy, Clone, Default)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
