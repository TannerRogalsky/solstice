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
    pub fn transform_point(&self, x: f32, y: f32) -> (f32, f32) {
        (
            x * self.scale_x + y * self.skew_x + self.translation_x,
            x * self.skew_y + y * self.scale_y + self.translation_y,
        )
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
