mod box_geometry;
mod plane_geometry;
mod polyhedron_geometry;
mod sphere_geometry;

pub use box_geometry::Box;
pub use plane_geometry::Plane;
pub use polyhedron_geometry::Polyhedron;
pub use sphere_geometry::Sphere;

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Default)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn lerp(&self, other: &Point3D, ratio: f32) -> Point3D {
        let mut r = *self;
        r.x += (other.x - self.x) * ratio;
        r.y += (other.y - self.y) * ratio;
        r.z += (other.z - self.z) * ratio;
        r
    }

    pub fn normalize(&self) -> Point3D {
        self.divide_scalar(self.length())
    }

    pub fn divide_scalar(&self, scalar: f32) -> Point3D {
        self.multiply_scalar(1.0 / scalar)
    }

    pub fn multiply_scalar(&self, scalar: f32) -> Point3D {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}
