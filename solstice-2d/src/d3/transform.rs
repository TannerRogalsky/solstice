use crate::Rad;
use nalgebra::{Isometry3, Translation3, UnitQuaternion, Vector3};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Transform3D {
    isometry: Isometry3<f32>,
    scale: Vector3<f32>,
}

impl Default for Transform3D {
    fn default() -> Self {
        Self {
            isometry: Isometry3::translation(0., 0., 0.),
            scale: Vector3::new(1., 1., 1.),
        }
    }
}

impl Transform3D {
    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        Self {
            isometry: Isometry3::translation(x, y, z),
            scale: Vector3::new(1., 1., 1.),
        }
    }

    pub fn rotation<R, P, Y>(roll: R, pitch: P, yaw: Y) -> Self
    where
        R: Into<Rad>,
        P: Into<Rad>,
        Y: Into<Rad>,
    {
        Self {
            isometry: Isometry3::from_parts(
                Translation3::new(0., 0., 0.),
                UnitQuaternion::from_euler_angles(roll.into().0, pitch.into().0, yaw.into().0),
            ),
            ..Default::default()
        }
    }

    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        Self {
            scale: Vector3::new(x, y, z),
            ..Default::default()
        }
    }

    pub fn lerp_slerp(&self, other: &Self, t: f32) -> Self {
        let isometry = self.isometry.lerp_slerp(&other.isometry, t);
        let scale = self.scale.lerp(&other.scale, t);
        Self { isometry, scale }
    }

    pub fn transform_point(&self, x: f32, y: f32, z: f32) -> [f32; 3] {
        let p = nalgebra::Point3::new(x * self.scale.x, y * self.scale.y, z * self.scale.z);
        let p = self.isometry.transform_point(&p);
        [p.x, p.y, p.z]
    }

    pub fn look_at(x: f32, y: f32, z: f32) -> Self {
        let eye = nalgebra::Point3::new(0., 0., 0.);
        let target = nalgebra::Point3::new(x, y, z);
        let up = nalgebra::Vector3::y();
        Self {
            isometry: Isometry3::look_at_lh(&eye, &target, &up),
            ..Default::default()
        }
    }
}

impl std::ops::Mul for Transform3D {
    type Output = Transform3D;

    fn mul(self, rhs: Self) -> Self::Output {
        let t = self
            .isometry
            .rotation
            .transform_vector(&rhs.isometry.translation.vector.component_mul(&self.scale))
            + self.isometry.translation.vector;
        Self {
            isometry: Isometry3::from_parts(
                t.into(),
                self.isometry.rotation * rhs.isometry.rotation,
            ),
            scale: self.scale.component_mul(&rhs.scale),
        }
    }
}

impl std::ops::MulAssign for Transform3D {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl From<Transform3D> for mint::ColumnMatrix4<f32> {
    fn from(inner: Transform3D) -> Self {
        inner
            .isometry
            .to_homogeneous()
            .prepend_nonuniform_scaling(&inner.scale)
            .into()
    }
}

impl From<&Transform3D> for mint::ColumnMatrix4<f32> {
    fn from(inner: &Transform3D) -> Self {
        inner
            .isometry
            .to_homogeneous()
            .prepend_nonuniform_scaling(&inner.scale)
            .into()
    }
}
