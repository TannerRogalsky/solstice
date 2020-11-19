use crate::Rad;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub struct Transform3D {
    pub inner: nalgebra::Matrix4<f32>,
}

impl Default for Transform3D {
    fn default() -> Self {
        Self {
            inner: nalgebra::Matrix4::identity(),
        }
    }
}

impl Transform3D {
    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        Self {
            inner: nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(x, y, z)),
        }
    }

    pub fn rotation<R: Into<Rad>>(roll: R, pitch: R, yaw: R) -> Self {
        Self {
            inner: nalgebra::Matrix4::from_euler_angles(
                roll.into().0,
                pitch.into().0,
                yaw.into().0,
            ),
        }
    }

    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        Self {
            inner: nalgebra::Matrix4::new_nonuniform_scaling(&nalgebra::Vector3::new(x, y, z)),
        }
    }
}

impl std::ops::Mul for Transform3D {
    type Output = Transform3D;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            inner: self.inner * rhs.inner,
        }
    }
}

impl std::ops::MulAssign for Transform3D {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl From<crate::Transform2D> for Transform3D {
    fn from(t: crate::Transform2D) -> Self {
        let t: mint::ColumnMatrix4<f32> = t.into();
        Self { inner: t.into() }
    }
}
