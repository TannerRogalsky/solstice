use crate::{Geometry, Rad, Vertex3D};

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct Sphere {
    pub radius: f32,
    pub width_segments: u32,
    pub height_segments: u32,
    pub phi_start: Rad,
    pub phi_length: Rad,
    pub theta_start: Rad,
    pub theta_length: Rad,
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            radius: 1.,
            width_segments: 8,
            height_segments: 6,
            phi_start: Rad(0.0),
            phi_length: Rad(std::f32::consts::PI * 2.),
            theta_start: Rad(0.0),
            theta_length: Rad(std::f32::consts::PI),
        }
    }
}

impl Sphere {
    fn vertices(&self) -> Vec<Vertex3D> {
        let mut vertices = vec![];
        let theta_end = std::f32::consts::PI.min(self.theta_start.0 + self.theta_length.0);

        for y in 0..=self.height_segments {
            let v = y as f32 / self.height_segments as f32;

            // special consideration for the poles
            let u_offset = if y == 0 && self.theta_start.0 == 0. {
                0.5 / self.width_segments as f32
            } else if y == self.height_segments && theta_end == std::f32::consts::PI {
                -0.5 / self.width_segments as f32
            } else {
                0.
            };

            for x in 0..=self.width_segments {
                let u = x as f32 / self.width_segments as f32;

                let position = nalgebra::Vector3::new(
                    -self.radius
                        * (self.phi_start.0 + u * self.phi_length.0).cos()
                        * (self.theta_start.0 + v * self.theta_length.0).sin(),
                    self.radius * (self.theta_start.0 + v * self.theta_length.0).cos(),
                    self.radius
                        * (self.phi_start.0 + u * self.phi_length.0).sin()
                        * (self.theta_start.0 + v * self.theta_length.0).sin(),
                );
                vertices.push(Vertex3D {
                    normal: position.normalize().into(),
                    position: position.into(),
                    uv: [u + u_offset, v],
                    color: [1., 1., 1., 1.],
                });
            }
        }

        vertices
    }

    fn indices(&self) -> Vec<u32> {
        let mut indices = vec![];
        let theta_end = std::f32::consts::PI.min(self.theta_start.0 + self.theta_length.0);

        let mut index = 0;
        let mut grid = vec![];
        for _y in 0..=self.height_segments {
            let mut vertices_row = vec![];
            for _x in 0..=self.width_segments {
                vertices_row.push(index);
                index += 1;
            }
            grid.push(vertices_row);
        }

        for iy in 0..self.height_segments as usize {
            for ix in 0..self.width_segments as usize {
                let a = grid[iy][ix + 1];
                let b = grid[iy][ix];
                let c = grid[iy + 1][ix];
                let d = grid[iy + 1][ix + 1];

                if iy as u32 != 0 || self.theta_start.0 > 0. {
                    indices.extend_from_slice(&[a, b, d]);
                }
                if iy as u32 != self.height_segments - 1 || theta_end < std::f32::consts::PI {
                    indices.extend_from_slice(&[b, c, d]);
                }
            }
        }

        indices
    }
}

impl From<&Sphere> for Geometry<'_, Vertex3D> {
    fn from(s: &Sphere) -> Self {
        Self::new(s.vertices(), Some(s.indices()))
    }
}

impl From<Sphere> for Geometry<'_, Vertex3D> {
    fn from(s: Sphere) -> Self {
        (&s).into()
    }
}
