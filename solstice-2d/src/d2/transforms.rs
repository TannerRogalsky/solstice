#[derive(Copy, Clone, PartialEq)]
pub struct Transform {
    pub translation_x: f32,
    pub translation_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub skew_x: f32,
    pub skew_y: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation_x: 0.,
            translation_y: 0.,
            scale_x: 1.,
            scale_y: 1.,
            skew_x: 0.,
            skew_y: 0.,
        }
    }
}

impl Transform {
    pub fn rotation<R: Into<super::Rad>>(phi: R) -> Self {
        let phi = phi.into();
        let (sin, cos) = phi.0.sin_cos();
        Self {
            scale_x: cos,
            skew_y: -sin,
            skew_x: sin,
            scale_y: cos,
            ..Default::default()
        }
    }

    pub fn scale(scale: f32) -> Self {
        Self {
            scale_x: scale,
            scale_y: scale,
            ..Default::default()
        }
    }

    pub fn translation(x: f32, y: f32) -> Self {
        Self {
            translation_x: x,
            translation_y: y,
            ..Default::default()
        }
    }

    pub fn transform_point(&self, x: f32, y: f32) -> (f32, f32) {
        (
            x * self.scale_x + y * self.skew_x + self.translation_x,
            x * self.skew_y + y * self.scale_y + self.translation_y,
        )
    }
}

impl std::ops::Mul for Transform {
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self::Output {
        let lhs = self;
        self.scale_x = lhs.scale_x * rhs.scale_x + lhs.skew_x * rhs.skew_y;
        self.skew_y = lhs.skew_y * rhs.scale_x + lhs.scale_y * rhs.skew_y;
        self.skew_x = lhs.scale_x * rhs.skew_x + lhs.skew_x * rhs.scale_y;
        self.scale_y = lhs.skew_y * rhs.skew_x + lhs.scale_y * rhs.scale_y;
        self.translation_x =
            lhs.scale_x * rhs.translation_x + lhs.skew_x * rhs.translation_y + lhs.translation_x;
        self.translation_y =
            lhs.skew_y * rhs.translation_x + lhs.scale_y * rhs.translation_y + lhs.translation_y;
        self
    }
}

impl std::ops::MulAssign for Transform {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Into<mint::ColumnMatrix3<f32>> for Transform {
    fn into(self) -> mint::ColumnMatrix3<f32> {
        mint::ColumnMatrix3 {
            x: mint::Vector3 {
                x: self.scale_x,
                y: self.skew_y,
                z: 0.,
            },
            y: mint::Vector3 {
                x: self.skew_x,
                y: self.scale_y,
                z: 0.,
            },
            z: mint::Vector3 {
                x: self.translation_x,
                y: self.translation_y,
                z: 1.,
            },
        }
    }
}

impl Into<mint::ColumnMatrix4<f32>> for Transform {
    fn into(self) -> mint::ColumnMatrix4<f32> {
        mint::ColumnMatrix4 {
            x: mint::Vector4 {
                x: self.scale_x,
                y: self.skew_y,
                z: 0.,
                w: 0.,
            },
            y: mint::Vector4 {
                x: self.scale_y,
                y: self.skew_x,
                z: 0.,
                w: 0.,
            },
            z: mint::Vector4 {
                x: 0.,
                y: 0.,
                z: 1.,
                w: 0.,
            },
            w: mint::Vector4 {
                x: self.translation_x,
                y: self.translation_y,
                z: 0.,
                w: 1.,
            },
        }
    }
}

#[derive(Default)]
pub struct Transforms {
    base: Transform,
    stack: Vec<Transform>,
}

impl Transforms {
    pub fn current(&self) -> &Transform {
        self.stack.last().unwrap_or(&self.base)
    }

    pub fn current_mut(&mut self) -> &mut Transform {
        self.stack.last_mut().unwrap_or(&mut self.base)
    }

    pub fn push(&mut self) {
        self.stack.push(*self.current())
    }

    pub fn pop(&mut self) -> Option<Transform> {
        self.stack.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn transform_point_identity() {
        let identity = Transform::default();

        let (px, py) = (0., 0.);
        assert_eq!(identity.transform_point(px, py), (px, py));

        let (px, py) = (100., 0.);
        assert_eq!(identity.transform_point(px, py), (px, py));

        let (px, py) = (-213., 123.);
        assert_eq!(identity.transform_point(px, py), (px, py));
    }

    #[test]
    pub fn transform_point_translation() {
        let identity = Transform {
            translation_x: 2.,
            translation_y: 1.5,
            ..Transform::default()
        };

        let (px, py) = (0., 0.);
        assert_eq!(identity.transform_point(px, py), (2., 1.5));
    }

    #[test]
    pub fn transform_point_rotation() {
        use approx::*;

        let transform = Transform::rotation(crate::Deg(90.));
        assert_abs_diff_eq!(transform.scale_x, 0.);
        assert_abs_diff_eq!(transform.scale_y, 0.);
        assert_abs_diff_eq!(transform.skew_x, 1.);
        assert_abs_diff_eq!(transform.skew_y, -1.);

        let (px, py) = (0., 0.);
        assert_eq!(transform.transform_point(px, py), (px, py));

        let (px, py) = (1., 0.);
        let (tx, ty) = transform.transform_point(px, py);
        assert_abs_diff_eq!(tx, 0.);
        assert_abs_diff_eq!(ty, -1.);

        let (tx, ty) = transform.transform_point(tx, ty);
        assert_abs_diff_eq!(tx, -px);
        assert_abs_diff_eq!(ty, -py);

        let (px, py) = (2., 2.);
        let (tx, ty) = transform.transform_point(px, py);
        assert_abs_diff_eq!(tx, 2., epsilon = 0.001);
        assert_abs_diff_eq!(ty, -2., epsilon = 0.001);

        let (tx, ty) = transform.transform_point(tx, ty);
        assert_abs_diff_eq!(tx, -2., epsilon = 0.001);
        assert_abs_diff_eq!(ty, -2., epsilon = 0.001);

        let (tx, ty) = transform.transform_point(tx, ty);
        assert_abs_diff_eq!(tx, -2., epsilon = 0.001);
        assert_abs_diff_eq!(ty, 2., epsilon = 0.001);

        let (tx, ty) = transform.transform_point(tx, ty);
        assert_abs_diff_eq!(tx, 2., epsilon = 0.001);
        assert_abs_diff_eq!(ty, 2., epsilon = 0.001);
    }

    #[test]
    pub fn transform_point_scale() {
        let identity = Transform {
            scale_x: 2.,
            scale_y: 1.5,
            ..Transform::default()
        };

        let (px, py) = (0., 0.);
        assert_eq!(identity.transform_point(px, py), (px, py));

        let (px, py) = (100., 0.);
        assert_eq!(identity.transform_point(px, py), (200., 0.));

        let (px, py) = (-213., 123.);
        assert_eq!(identity.transform_point(px, py), (px * 2., py * 1.5));
    }

    #[test]
    pub fn transform_point() {
        let identity = Transform {
            translation_x: 100.,
            translation_y: 200.,
            scale_x: 2.,
            scale_y: 1.5,
            ..Transform::default()
        };

        let (px, py) = (0., 0.);
        assert_eq!(identity.transform_point(px, py), (100., 200.));

        let (px, py) = (1., 2.);
        assert_eq!(identity.transform_point(px, py), (102., 203.));
    }
}
