use crate::{
    d2::{SimpleConvexGeometry, Vertex2D},
    Geometry,
};

/// An angle, in radians.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Rad(pub f32);

/// An angle, in degrees.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
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

#[derive(Debug, Copy, Clone, PartialEq)]
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

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Arc {
    pub arc_type: ArcType,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub angle1: Rad,
    pub angle2: Rad,
    pub segments: u32,
}

impl SimpleConvexGeometry for Arc {
    type Vertices = std::vec::IntoIter<Vertex2D>;

    fn vertices(&self) -> Self::Vertices {
        let Arc {
            arc_type,
            x,
            y,
            radius,
            angle1,
            angle2,
            segments,
        } = *self;
        let (angle1, angle2) = (angle1.0, angle2.0);

        if segments == 0 || (angle1 - angle2).abs() < f32::EPSILON {
            return Vec::<Vertex2D>::new().into_iter();
        }

        const TWO_PI: f32 = std::f32::consts::PI * 2.;
        if (angle1 - angle2).abs() >= TWO_PI {
            return SimpleConvexGeometry::vertices(&Circle {
                x,
                y,
                radius,
                segments,
            })
            .collect::<Vec<_>>() // type constraints require that we collect here
            .into_iter();
        }

        let angle_shift = (angle2 - angle1) / segments as f32;
        if angle_shift == 0. {
            return Vec::<Vertex2D>::new().into_iter(); // bail on precision fail
        }

        let mut create_points = {
            let mut phi = angle1;
            move |coordinates: &mut [Vertex2D]| {
                for coordinate in coordinates.iter_mut() {
                    let (s, c) = phi.sin_cos();
                    let x = x + radius * c;
                    let y = y + radius * s;
                    coordinate.position[0] = x;
                    coordinate.position[1] = y;
                    coordinate.uv[0] = (c + 1.) / 2.;
                    coordinate.uv[1] = (s + 1.) / 2.;
                    phi += angle_shift;
                }
            }
        };

        let vertices = match arc_type {
            ArcType::Pie => {
                let num_coords = segments as usize + 3;
                let mut coords = vec![Vertex2D::default(); num_coords];
                let anchor = Vertex2D {
                    position: [x, y],
                    ..Vertex2D::default()
                };
                coords[0] = anchor;
                coords[num_coords - 1] = anchor;
                create_points(&mut coords[1..num_coords - 1]);
                coords
            }
            ArcType::Open => {
                let num_coords = segments as usize + 1;
                let mut coords = vec![Vertex2D::default(); num_coords];
                create_points(&mut coords);
                coords
            }
            ArcType::Closed => {
                let num_coords = segments as usize + 2;
                let mut coords = vec![Vertex2D::default(); num_coords];
                create_points(&mut coords);
                coords[num_coords - 1] = coords[0];
                coords
            }
        };

        vertices.into_iter()
    }

    fn vertex_count(&self) -> usize {
        match self.arc_type {
            ArcType::Pie => self.segments as usize + 3,
            ArcType::Open => self.segments as usize + 1,
            ArcType::Closed => self.segments as usize + 2,
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Circle {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub segments: u32,
}

impl SimpleConvexGeometry for Circle {
    type Vertices = <SimpleConvexPolygon as SimpleConvexGeometry>::Vertices;

    fn vertices(&self) -> Self::Vertices {
        let ellipse: Ellipse = (*self).into();
        let polygon: SimpleConvexPolygon = ellipse.into();
        SimpleConvexGeometry::vertices(&polygon)
    }

    fn vertex_count(&self) -> usize {
        self.segments as usize
    }
}

impl From<Circle> for Ellipse {
    fn from(c: Circle) -> Self {
        Self {
            x: c.x,
            y: c.y,
            radius_x: c.radius,
            radius_y: c.radius,
            segments: c.segments,
        }
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Ellipse {
    pub x: f32,
    pub y: f32,
    pub radius_x: f32,
    pub radius_y: f32,
    pub segments: u32,
}

impl From<Ellipse> for SimpleConvexPolygon {
    fn from(e: Ellipse) -> Self {
        Self {
            x: e.x,
            y: e.y,
            vertex_count: e.segments,
            radius_x: e.radius_x,
            radius_y: e.radius_y,
        }
    }
}

impl SimpleConvexGeometry for Ellipse {
    type Vertices = <SimpleConvexPolygon as SimpleConvexGeometry>::Vertices;

    fn vertices(&self) -> Self::Vertices {
        let polygon: SimpleConvexPolygon = (*self).into();
        SimpleConvexGeometry::vertices(&polygon)
    }

    fn vertex_count(&self) -> usize {
        self.segments as usize
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn vertices(&self) -> Vec<Vertex2D> {
        vec![
            Vertex2D {
                position: [self.x, self.y],
                uv: [0., 0.],
                ..Default::default()
            },
            Vertex2D {
                position: [self.x, self.y + self.height],
                uv: [0., 1.],
                ..Default::default()
            },
            Vertex2D {
                position: [self.x + self.width, self.y + self.height],
                uv: [1., 1.],
                ..Default::default()
            },
            Vertex2D {
                position: [self.x + self.width, self.y],
                uv: [1., 0.],
                ..Default::default()
            },
        ]
    }
}

impl From<Rectangle> for solstice::quad_batch::Quad<Vertex2D> {
    fn from(r: Rectangle) -> Self {
        use solstice::{quad_batch::Quad, viewport::Viewport};
        let positions = Quad::from(Viewport::new(r.x, r.y, r.width, r.height));
        let uvs = Quad::from(Viewport::new(0., 0., 1., 1.));
        positions.zip(uvs).map(|((x, y), (s, t))| Vertex2D {
            position: [x, y],
            uv: [s, t],
            ..Default::default()
        })
    }
}

impl From<Rectangle> for solstice::quad_batch::Quad<(f32, f32)> {
    fn from(r: Rectangle) -> Self {
        use solstice::{quad_batch::Quad, viewport::Viewport};
        Quad::from(Viewport::new(r.x, r.y, r.width, r.height))
    }
}

impl From<&Rectangle> for Geometry<'_, Vertex2D> {
    fn from(r: &Rectangle) -> Self {
        Geometry::new(r.vertices(), Some(&[0u32, 1, 2, 0, 3, 2][..]))
    }
}

impl From<Rectangle> for Geometry<'_, Vertex2D> {
    fn from(r: Rectangle) -> Self {
        (&r).into()
    }
}

impl From<solstice::quad_batch::Quad<Vertex2D>> for Geometry<'_, Vertex2D> {
    fn from(quad: solstice::quad_batch::Quad<Vertex2D>) -> Self {
        Geometry::new(
            quad.vertices.to_vec(),
            Some(
                solstice::quad_batch::INDICES[..]
                    .iter()
                    .copied()
                    .map(|i| i as u32)
                    .collect::<Vec<_>>(),
            ),
        )
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct RegularPolygon {
    pub x: f32,
    pub y: f32,
    pub vertex_count: u32,
    pub radius: f32,
}

impl RegularPolygon {
    pub fn new(x: f32, y: f32, vertex_count: u32, radius: f32) -> Self {
        Self {
            x,
            y,
            vertex_count,
            radius,
        }
    }
}

impl SimpleConvexGeometry for RegularPolygon {
    type Vertices = std::iter::Map<std::ops::Range<u32>, Box<dyn Fn(u32) -> Vertex2D>>;

    fn vertices(&self) -> Self::Vertices {
        const TWO_PI: f32 = std::f32::consts::PI * 2.;
        let RegularPolygon {
            x,
            y,
            vertex_count,
            radius,
        } = *self;
        let angle_shift = TWO_PI / vertex_count as f32;

        // an allocation is a small price to pay for my sanity
        (0..vertex_count).map(Box::new(move |i| {
            let phi = angle_shift * i as f32;
            let (s, c) = phi.sin_cos();
            let (x, y) = (x + radius * c, y + radius * s);
            Vertex2D::new([x, y], [1., 1., 1., 1.], [(c + 1.) / 2., (s + 1.) / 2.])
        }))
    }

    fn vertex_count(&self) -> usize {
        self.vertex_count as _
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct SimpleConvexPolygon {
    pub x: f32,
    pub y: f32,
    pub vertex_count: u32,
    pub radius_x: f32,
    pub radius_y: f32,
}

impl SimpleConvexGeometry for SimpleConvexPolygon {
    type Vertices = std::iter::Map<std::ops::Range<u32>, Box<dyn Fn(u32) -> Vertex2D>>;

    fn vertices(&self) -> Self::Vertices {
        const TWO_PI: f32 = std::f32::consts::PI * 2.;
        let SimpleConvexPolygon {
            x,
            y,
            vertex_count,
            radius_x,
            radius_y,
        } = *self;
        let angle_shift = TWO_PI / vertex_count as f32;

        // an allocation is a small price to pay for my sanity
        (0..vertex_count).map(Box::new(move |i| {
            let phi = angle_shift * i as f32;
            let (x, y) = (x + radius_x * phi.cos(), y + radius_y * phi.sin());
            Vertex2D::new([x, y], [1., 1., 1., 1.], [0.5, 0.5])
        }))
    }

    fn vertex_count(&self) -> usize {
        self.vertex_count as usize
    }
}

impl<T> From<T> for Geometry<'_, Vertex2D>
where
    T: SimpleConvexGeometry,
{
    fn from(s: T) -> Self {
        let vertices = s.vertices().collect::<Vec<_>>();
        let indices = (1..(vertices.len() as u32).saturating_sub(1))
            .flat_map(|i| [0, i, i + 1])
            .collect::<Vec<_>>();
        Geometry::new(vertices, Some(indices))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry() {
        use crate::Geometry;
        let geometry = SimpleConvexPolygon {
            x: 0.0,
            y: 0.0,
            vertex_count: 3,
            radius_x: 100.,
            radius_y: 100.,
        };

        let data = Geometry::<'_, Vertex2D>::from(geometry);
        assert_eq!(data.vertices.len(), 3);
        assert_eq!(data.indices.unwrap().len(), 3);

        let geometry = SimpleConvexPolygon {
            vertex_count: 4,
            ..geometry
        };
        let data = Geometry::<'_, Vertex2D>::from(geometry);
        assert_eq!(data.vertices.len(), 4);
        assert_eq!(data.indices.unwrap().len(), 6);

        let geometry = SimpleConvexPolygon {
            vertex_count: 5,
            ..geometry
        };
        let data = Geometry::<'_, Vertex2D>::from(geometry);
        assert_eq!(data.vertices.len(), 5);
        assert_eq!(data.indices.unwrap().len(), (5 - 2) * 3);
    }
}
