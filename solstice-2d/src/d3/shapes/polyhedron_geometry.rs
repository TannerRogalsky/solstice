use crate::d3::{Point3D, Vertex3D};
use crate::Geometry;

#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub struct Polyhedron {
    pub vertices: Vec<Point3D>,
    pub indices: Vec<u32>,
    pub radius: f32,
    pub detail: u32,
}

impl Polyhedron {
    pub fn tetrahedron(radius: f32, detail: u32) -> Self {
        let vertices = vec![
            Point3D::new(1., 1., 1.),
            Point3D::new(-1., -1., 1.),
            Point3D::new(-1., 1., -1.),
            Point3D::new(1., -1., -1.),
        ];
        let indices = vec![2, 1, 0, 0, 3, 2, 1, 3, 0, 2, 3, 1];
        Self {
            vertices,
            indices,
            radius,
            detail,
        }
    }

    pub fn octahedron(radius: f32, detail: u32) -> Self {
        let vertices = vec![
            Point3D::new(1., 0., 0.),
            Point3D::new(-1., 0., 0.),
            Point3D::new(0., 1., 0.),
            Point3D::new(0., -1., 0.),
            Point3D::new(0., 0., 1.),
            Point3D::new(0., 0., -1.),
        ];

        let indices = vec![
            0, 2, 4, 0, 4, 3, 0, 3, 5, 0, 5, 2, 1, 2, 5, 1, 5, 3, 1, 3, 4, 1, 4, 2,
        ];

        Self {
            vertices,
            indices,
            radius,
            detail,
        }
    }

    pub fn icosahedron(radius: f32, detail: u32) -> Self {
        let t = (1.0 + 5f32.sqrt()) / 2.0;

        let vertices = vec![
            Point3D::new(-1., t, 0.),
            Point3D::new(1., t, 0.),
            Point3D::new(-1., -t, 0.),
            Point3D::new(1., -t, 0.),
            Point3D::new(0., -1., t),
            Point3D::new(0., 1., t),
            Point3D::new(0., -1., -t),
            Point3D::new(0., 1., -t),
            Point3D::new(t, 0., -1.),
            Point3D::new(t, 0., 1.),
            Point3D::new(-t, 0., -1.),
            Point3D::new(-t, 0., 1.),
        ];

        let indices = vec![
            0, 11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 0, 10, 11, 1, 5, 9, 5, 11, 4, 11, 10, 2, 10, 7,
            6, 7, 1, 8, 3, 9, 4, 3, 4, 2, 3, 2, 6, 3, 6, 8, 3, 8, 9, 4, 9, 5, 2, 4, 11, 6, 2, 10,
            8, 6, 7, 9, 8, 1,
        ];

        Self {
            vertices,
            indices,
            radius,
            detail,
        }
    }

    pub fn dodecahedron(radius: f32, detail: u32) -> Self {
        let t = (1. + 5f32.sqrt()) / 2.;
        let r = 1. / t;

        let vertices = vec![
            // (±1, ±1, ±1)
            Point3D::new(-1.0, -1.0, -1.0),
            Point3D::new(-1.0, -1.0, 1.0),
            Point3D::new(-1.0, 1.0, -1.0),
            Point3D::new(-1.0, 1.0, 1.0),
            Point3D::new(1.0, -1.0, -1.0),
            Point3D::new(1.0, -1.0, 1.0),
            Point3D::new(1.0, 1.0, -1.0),
            Point3D::new(1.0, 1.0, 1.0),
            // (0, ±1/φ, ±φ)
            Point3D::new(0.0, -r, -t),
            Point3D::new(0.0, -r, t),
            Point3D::new(0.0, r, -t),
            Point3D::new(0.0, r, t),
            // (±1/φ, ±φ, 0)
            Point3D::new(-r, -t, 0.0),
            Point3D::new(-r, t, 0.0),
            Point3D::new(r, -t, 0.0),
            Point3D::new(r, t, 0.0),
            // (±φ, 0, ±1/φ)
            Point3D::new(-t, 0.0, -r),
            Point3D::new(t, 0.0, -r),
            Point3D::new(-t, 0.0, r),
            Point3D::new(t, 0.0, r),
        ];

        let indices = vec![
            3, 11, 7, 3, 7, 15, 3, 15, 13, 7, 19, 17, 7, 17, 6, 7, 6, 15, 17, 4, 8, 17, 8, 10, 17,
            10, 6, 8, 0, 16, 8, 16, 2, 8, 2, 10, 0, 12, 1, 0, 1, 18, 0, 18, 16, 6, 10, 2, 6, 2, 13,
            6, 13, 15, 2, 16, 18, 2, 18, 3, 2, 3, 13, 18, 1, 9, 18, 9, 11, 18, 11, 3, 4, 14, 12, 4,
            12, 0, 4, 0, 8, 11, 9, 5, 11, 5, 19, 11, 19, 7, 19, 5, 14, 19, 14, 4, 19, 4, 17, 1, 12,
            14, 1, 14, 5, 1, 5, 9,
        ];

        Self {
            vertices,
            indices,
            radius,
            detail,
        }
    }

    fn vertex_count(&self) -> usize {
        self.indices.len() * ((self.detail as usize + 1).pow(2))
    }
}

fn subdivide(detail: u32, vertices: &mut Vec<Point3D>, indices: &[u32], v: &[Point3D]) {
    for i in indices.chunks_exact(3) {
        let a = v[i[0] as usize];
        let b = v[i[1] as usize];
        let c = v[i[2] as usize];

        subdivide_face(a, b, c, detail, vertices);
    }
}

fn subdivide_face(a: Point3D, b: Point3D, c: Point3D, detail: u32, vertices: &mut Vec<Point3D>) {
    let cols = (detail + 1) as usize;
    let mut v = Vec::<Vec<Point3D>>::with_capacity(cols);

    for col in 0..=cols {
        let aj = a.lerp(&c, col as f32 / cols as f32);
        let bj = b.lerp(&c, col as f32 / cols as f32);

        let rows = cols - col;
        v.push(
            (0..=rows)
                .map(|row| {
                    if row == 0 && col == cols {
                        aj
                    } else {
                        aj.lerp(&bj, row as f32 / rows as f32)
                    }
                })
                .collect(),
        );
    }

    for col in 0..cols {
        for row in 0..(2 * (cols - col) - 1) {
            let k = row / 2; // a flooring division
            if row % 2 == 0 {
                vertices.push(v[col][k + 1]);
                vertices.push(v[col + 1][k]);
                vertices.push(v[col][k]);
            } else {
                vertices.push(v[col][k + 1]);
                vertices.push(v[col + 1][k + 1]);
                vertices.push(v[col + 1][k]);
            }
        }
    }
}

fn apply_radius(radius: f32, vertices: &mut [Point3D]) {
    for vertex in vertices.iter_mut() {
        *vertex = vertex.normalize().multiply_scalar(radius);
    }
}

fn azimuth(point: &Point3D) -> f32 {
    point.z.atan2(-point.x)
}

fn inclination(point: &Point3D) -> f32 {
    (-point.y).atan2((point.x * point.x + point.z * point.z).sqrt())
}

fn correct_uvs(vertices: &mut Vec<Vertex3D>) {
    for triangle in vertices.chunks_exact_mut(3) {
        let a = triangle[0].position;
        let b = triangle[1].position;
        let c = triangle[2].position;

        let cx = a[0] + b[0] + c[0];
        let cy = a[1] + b[1] + c[1];
        let cz = a[2] + b[2] + c[2];
        let centroid = Point3D::new(cx, cy, cz).divide_scalar(3.);
        let azi = azimuth(&centroid);

        correct_uv(&mut triangle[0].uv[0], &a, azi);
        correct_uv(&mut triangle[1].uv[0], &b, azi);
        correct_uv(&mut triangle[2].uv[0], &c, azi);
    }
}

fn correct_uv(uv: &mut f32, vector: &[f32; 3], azi: f32) {
    if (azi < 0.) && (*uv - 1.).abs() < f32::EPSILON {
        *uv = *uv - 1.;
    }

    if (vector[0] == 0.) && (vector[2] == 0.) {
        *uv = azi / 2. / std::f32::consts::PI + 0.5;
    }
}

fn correct_seam(vertices: &mut Vec<Vertex3D>) {
    for triangle in vertices.chunks_exact_mut(3) {
        let x0 = triangle[0].uv[0];
        let x1 = triangle[1].uv[0];
        let x2 = triangle[2].uv[0];

        let max = x0.max(x1.max(x2));
        let min = x0.min(x1.min(x2));

        if max > 0.9 && min < 0.1 {
            if x0 < 0.2 {
                triangle[0].uv[0] += 1.;
            }
            if x1 < 0.2 {
                triangle[1].uv[0] += 1.;
            }
            if x2 < 0.2 {
                triangle[2].uv[0] += 1.;
            }
        }
    }
}

impl From<&Polyhedron> for Geometry<'_, Vertex3D> {
    fn from(p: &Polyhedron) -> Self {
        let mut vertices = Vec::with_capacity(p.vertex_count());

        subdivide(
            p.detail,
            &mut vertices,
            p.indices.as_slice(),
            p.vertices.as_slice(),
        );
        apply_radius(p.radius, vertices.as_mut_slice());

        let mut vertices = vertices
            .into_iter()
            .map(|p| {
                let u = azimuth(&p) / 2. / std::f32::consts::PI + 0.5;
                let v = inclination(&p) / std::f32::consts::PI + 0.5;
                let normal = p.normalize();
                Vertex3D {
                    position: [p.x, p.y, p.z],
                    uv: [u, v],
                    color: [1., 1., 1., 1.],
                    normal: [normal.x, normal.y, normal.z],
                }
            })
            .collect::<Vec<_>>();

        correct_uvs(&mut vertices);
        correct_seam(&mut vertices);

        Self {
            vertices: vertices.into(),
            indices: None,
        }
    }
}

impl From<Polyhedron> for Geometry<'_, Vertex3D> {
    fn from(p: Polyhedron) -> Self {
        (&p).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subdivision_count() {
        for detail in 0..10 {
            let shape = Polyhedron::tetrahedron(1., detail);

            assert_eq!(12 * ((detail as usize + 1).pow(2)), shape.vertex_count());
        }
    }
}
