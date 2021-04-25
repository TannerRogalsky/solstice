use crate::{Geometry, Point3D, Sphere, Vertex3D};

/// Differs from a Sphere in that when "open" on an axis, additional geometry si generated creating
/// sides connected to the centroid.
#[derive(Clone, Debug, PartialEq)]
pub struct Arc3D(pub Sphere);

impl Arc3D {
    pub fn new(sphere: Sphere) -> Self {
        Self(sphere)
    }

    fn vertices(&self) -> Vec<Vertex3D> {
        let Sphere {
            radius,
            width_segments,
            height_segments,
            phi_start,
            phi_length,
            theta_start,
            theta_length,
        } = self.0;
        let mut vertices = vec![];
        let theta_end = std::f32::consts::PI.min(theta_start.0 + theta_length.0);

        for y in 0..=height_segments {
            let v = y as f32 / height_segments as f32;

            vertices.push(Vertex3D {
                normal: [0., v, 0.],
                position: [0., radius * (theta_start.0 + v * theta_length.0).cos(), 0.],
                uv: [0., v],
                color: [1., 1., 1., 1.],
            });

            // special consideration for the poles
            let u_offset = if y == 0 && theta_start.0 == 0. {
                0.5 / width_segments as f32
            } else if y == height_segments && theta_end == std::f32::consts::PI {
                -0.5 / width_segments as f32
            } else {
                0.
            };

            for x in 0..=width_segments {
                let u = x as f32 / width_segments as f32;

                let position = Point3D::from([
                    -radius
                        * (phi_start.0 + u * phi_length.0).cos()
                        * (theta_start.0 + v * theta_length.0).sin(),
                    radius * (theta_start.0 + v * theta_length.0).cos(),
                    radius
                        * (phi_start.0 + u * phi_length.0).sin()
                        * (theta_start.0 + v * theta_length.0).sin(),
                ]);
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
        let Sphere {
            width_segments,
            height_segments,
            theta_start,
            theta_length,
            ..
        } = self.0;
        let mut indices = vec![];
        let theta_end = std::f32::consts::PI.min(theta_start.0 + theta_length.0);

        let mut index = 0;
        let mut grid = vec![];
        for _y in 0..=height_segments {
            let mut vertices_row = vec![];
            vertices_row.push(index);
            index += 1;
            for _x in 0..=width_segments {
                vertices_row.push(index);
                index += 1;
            }
            vertices_row.push(index);
            grid.push(vertices_row);
        }

        for iy in 0..height_segments as usize {
            for ix in 0..(width_segments + 2) as usize {
                let a = grid[iy][ix + 1];
                let b = grid[iy][ix];
                let c = grid[iy + 1][ix];
                let d = grid[iy + 1][ix + 1];

                if iy as u32 != 0 || theta_start.0 > 0. {
                    indices.extend_from_slice(&[a, b, d]);
                }
                if iy as u32 != height_segments - 1 || theta_end < std::f32::consts::PI {
                    indices.extend_from_slice(&[b, c, d]);
                }
            }
        }

        indices
    }
}

impl From<&Arc3D> for Geometry<'_, Vertex3D> {
    fn from(arc: &Arc3D) -> Self {
        Self::new(arc.vertices(), Some(arc.indices()))
    }
}

impl From<Arc3D> for Geometry<'_, Vertex3D> {
    fn from(arc: Arc3D) -> Self {
        (&arc).into()
    }
}
