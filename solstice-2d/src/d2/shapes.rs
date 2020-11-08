use super::{Geometry, SimpleConvexGeometry, Vertex2D};

/// An angle, in radians.
#[derive(Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Rad(pub f32);

/// An angle, in degrees.
#[derive(Copy, Clone, PartialEq, PartialOrd, Default)]
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

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone, Default)]
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

        if segments == 0 || (angle1 - angle2).abs() < std::f32::EPSILON {
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
                    phi += angle_shift;
                    let x = x + radius * phi.cos();
                    let y = y + radius * phi.sin();
                    coordinate.position[0] = x;
                    coordinate.position[1] = y;
                }
            }
        };

        let vertices = match arc_type {
            ArcType::Pie => {
                let num_coords = segments as usize + 3;
                let mut coords = vec![Vertex2D::default(); num_coords];
                coords[0] = Vertex2D {
                    position: [x, y],
                    ..Vertex2D::default()
                };
                create_points(&mut coords[1..]);
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

#[derive(Copy, Clone, Default)]
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

impl Into<Ellipse> for Circle {
    fn into(self) -> Ellipse {
        Ellipse {
            x: self.x,
            y: self.y,
            radius_x: self.radius,
            radius_y: self.radius,
            segments: self.segments,
        }
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Ellipse {
    pub x: f32,
    pub y: f32,
    pub radius_x: f32,
    pub radius_y: f32,
    pub segments: u32,
}

impl Into<SimpleConvexPolygon> for Ellipse {
    fn into(self) -> SimpleConvexPolygon {
        SimpleConvexPolygon {
            x: self.x,
            y: self.y,
            vertex_count: self.segments,
            radius_x: self.radius_x,
            radius_y: self.radius_y,
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

#[derive(Copy, Clone, Default, Debug)]
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
}

impl Geometry for Rectangle {
    type Vertices = std::vec::IntoIter<Vertex2D>;
    type Indices = std::iter::Copied<std::slice::Iter<'static, u32>>;

    fn vertices(&self) -> Self::Vertices {
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
        .into_iter()
    }

    fn indices(&self) -> Self::Indices {
        [0, 1, 2, 0, 3, 2].iter().copied()
    }
}

#[derive(Copy, Clone, Default, Debug)]
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

        assert_eq!(Geometry::vertices(&geometry).count(), 3);
        assert_eq!(Geometry::indices(&geometry).count(), 3);

        let geometry = SimpleConvexPolygon {
            vertex_count: 4,
            ..geometry
        };
        assert_eq!(Geometry::vertices(&geometry).count(), 4);
        assert_eq!(Geometry::indices(&geometry).count(), 6);

        let geometry = SimpleConvexPolygon {
            vertex_count: 5,
            ..geometry
        };
        assert_eq!(Geometry::vertices(&geometry).count(), 5);
        assert_eq!(Geometry::indices(&geometry).count(), (5 - 2) * 3);
    }
}
