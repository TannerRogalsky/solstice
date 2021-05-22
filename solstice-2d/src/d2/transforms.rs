use nalgebra::{Isometry2, Vector2};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Transform2D {
    pub isometry: Isometry2<f32>,
    pub scale: Vector2<f32>,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            isometry: Isometry2::identity(),
            scale: Vector2::new(1., 1.),
        }
    }
}

impl Transform2D {
    pub fn rotation<R: Into<super::Rad>>(rotation: R) -> Self {
        Self {
            isometry: Isometry2::rotation(-rotation.into().0),
            ..Default::default()
        }
    }

    pub fn scale(x: f32, y: f32) -> Self {
        Self {
            scale: Vector2::new(x, y),
            ..Default::default()
        }
    }

    pub fn translation(x: f32, y: f32) -> Self {
        Self {
            isometry: Isometry2::translation(x, y),
            ..Default::default()
        }
    }

    pub fn lerp_slerp(&self, other: &Self, t: f32) -> Self {
        let isometry = self.isometry.lerp_slerp(&other.isometry, t);
        let scale = self.scale.lerp(&other.scale, t);
        Self { isometry, scale }
    }

    pub fn transform_point(&self, x: f32, y: f32) -> [f32; 2] {
        let p = nalgebra::Point2::new(x * self.scale.x, y * self.scale.y);
        let p = self.isometry.transform_point(&p);
        [p.x, p.y]
    }
}

impl std::ops::Mul for Transform2D {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let t = self
            .isometry
            .rotation
            .transform_vector(&rhs.isometry.translation.vector.component_mul(&self.scale))
            + self.isometry.translation.vector;
        Self {
            isometry: Isometry2::from_parts(
                t.into(),
                self.isometry.rotation * rhs.isometry.rotation,
            ),
            scale: self.scale.component_mul(&rhs.scale),
        }
    }
}

impl std::ops::MulAssign for Transform2D {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl From<Transform2D> for mint::ColumnMatrix3<f32> {
    fn from(t: Transform2D) -> Self {
        t.isometry
            .to_homogeneous()
            .prepend_nonuniform_scaling(&t.scale)
            .into()
    }
}

impl From<Transform2D> for mint::ColumnMatrix4<f32> {
    fn from(t: Transform2D) -> Self {
        crate::Transform3D::from(t).into()
    }
}

impl From<Transform2D> for crate::Transform3D {
    fn from(t: Transform2D) -> Self {
        let translation = t.isometry.translation.vector;
        let rotation = t.isometry.rotation.angle();
        let scale = t.scale;
        Self::translation(translation.x, translation.y, 0.)
            * Self::rotation(crate::Rad(0.), crate::Rad(0.), crate::Rad(-rotation))
            * Self::scale(scale.x, scale.y, 1.)
    }
}

#[derive(Default)]
pub struct Transforms {
    base: Transform2D,
    stack: Vec<Transform2D>,
}

impl Transforms {
    pub fn current(&self) -> &Transform2D {
        self.stack.last().unwrap_or(&self.base)
    }

    pub fn current_mut(&mut self) -> &mut Transform2D {
        self.stack.last_mut().unwrap_or(&mut self.base)
    }

    pub fn push(&mut self) -> &mut Transform2D {
        self.stack.push(*self.current());
        self.current_mut()
    }

    pub fn pop(&mut self) -> Option<Transform2D> {
        self.stack.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rad;

    #[test]
    pub fn transform_point_identity() {
        let identity = Transform2D::default();

        let (px, py) = (0., 0.);
        assert_eq!(identity.transform_point(px, py), [px, py]);

        let (px, py) = (100., 0.);
        assert_eq!(identity.transform_point(px, py), [px, py]);

        let (px, py) = (-213., 123.);
        assert_eq!(identity.transform_point(px, py), [px, py]);
    }

    #[test]
    pub fn transform_point_translation() {
        let identity = Transform2D::translation(2., 1.5);

        let (px, py) = (0., 0.);
        assert_eq!(identity.transform_point(px, py), [2., 1.5]);
    }

    #[test]
    pub fn transform_point_rotation() {
        use approx::*;

        let transform = Transform2D::rotation(crate::Deg(90.));

        let (px, py) = (0., 0.);
        assert_eq!(transform.transform_point(px, py), [px, py]);

        let (px, py) = (1., 0.);
        let [tx, ty] = transform.transform_point(px, py);
        assert_abs_diff_eq!([0., -1.], [tx, ty]);

        let [tx, ty] = transform.transform_point(tx, ty);
        assert_abs_diff_eq!(tx, -px);
        assert_abs_diff_eq!(ty, -py);

        let (px, py) = (2., 2.);
        let [tx, ty] = transform.transform_point(px, py);
        assert_abs_diff_eq!(tx, 2., epsilon = 0.001);
        assert_abs_diff_eq!(ty, -2., epsilon = 0.001);

        let [tx, ty] = transform.transform_point(tx, ty);
        assert_abs_diff_eq!(tx, -2., epsilon = 0.001);
        assert_abs_diff_eq!(ty, -2., epsilon = 0.001);

        let [tx, ty] = transform.transform_point(tx, ty);
        assert_abs_diff_eq!(tx, -2., epsilon = 0.001);
        assert_abs_diff_eq!(ty, 2., epsilon = 0.001);

        let [tx, ty] = transform.transform_point(tx, ty);
        assert_abs_diff_eq!(tx, 2., epsilon = 0.001);
        assert_abs_diff_eq!(ty, 2., epsilon = 0.001);
    }

    #[test]
    pub fn transform_point_scale() {
        let identity = Transform2D::scale(2., 1.5);

        let (px, py) = (0., 0.);
        assert_eq!(identity.transform_point(px, py), [px, py]);

        let (px, py) = (100., 0.);
        assert_eq!(identity.transform_point(px, py), [200., 0.]);

        let (px, py) = (-213., 123.);
        assert_eq!(identity.transform_point(px, py), [px * 2., py * 1.5]);
    }

    #[test]
    pub fn transform_point() {
        let identity = Transform2D::translation(100., 200.) * Transform2D::scale(2., 1.5);

        let (px, py) = (0., 0.);
        assert_eq!(identity.transform_point(px, py), [100., 200.]);

        let (px, py) = (1., 2.);
        assert_eq!(identity.transform_point(px, py), [102., 203.]);
    }

    #[test]
    fn transform_mul() {
        use approx::*;

        let t1 = Transform2D::translation(1., 1.);
        let t2 = Transform2D::rotation(crate::Deg(90.));

        assert_abs_diff_eq!([1., 1.], t1.transform_point(0., 0.));
        assert_abs_diff_eq!([1., -1.], t2.transform_point(1., 1.));
        assert_abs_diff_eq!([1., -1.], (t2 * t1).transform_point(0., 0.));
    }

    #[test]
    fn conversion() {
        use crate::Transform3D;

        let t2_1 = Transform2D::translation(1., 2.);
        let t3_1 = Transform3D::translation(1., 2., 0.);

        assert_eq!(
            mint::ColumnMatrix4::<f32>::from(t2_1),
            mint::ColumnMatrix4::<f32>::from(t3_1)
        );

        let t2_2 = Transform2D::rotation(Rad(std::f32::consts::FRAC_PI_2));
        let t3_2 = Transform3D::rotation(Rad(0.), Rad(0.), Rad(std::f32::consts::FRAC_PI_2));

        assert_eq!(
            mint::ColumnMatrix4::<f32>::from(t2_2),
            mint::ColumnMatrix4::<f32>::from(t3_2)
        );

        assert_eq!(
            mint::ColumnMatrix4::<f32>::from(t2_1 * t2_2),
            mint::ColumnMatrix4::<f32>::from(t3_1 * t3_2)
        );
    }
}
