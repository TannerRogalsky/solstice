mod canvas;
mod shapes;
pub mod text;
mod transforms;
mod vertex;

pub use canvas::Canvas;
pub use glyph_brush::FontId;
pub use shapes::*;
pub use transforms::*;
pub use vertex::{Point, Vertex2D};

use super::{
    Color, Command, Draw, DrawList, DrawState, Geometry, GeometryVariants, LineState, LineVertex,
    Projection,
};
use solstice::texture::Texture;

impl<'a, G> Draw<crate::Vertex2D, G> for DrawList<'a>
where
    G: crate::GeometryKind<'a, crate::Vertex2D> + 'a,
{
    fn draw(&mut self, geometry: G) {
        self.push_draw(
            GeometryVariants::D2(geometry.into()),
            self.color,
            self.transform,
            None,
        );
    }

    fn draw_with_transform<TX>(&mut self, geometry: G, transform: TX)
    where
        TX: Into<mint::ColumnMatrix4<f32>>,
    {
        self.push_draw(
            GeometryVariants::D2(geometry.into()),
            self.color,
            transform.into(),
            None,
        );
    }

    fn draw_with_color<C: Into<Color>>(&mut self, geometry: G, color: C) {
        self.push_draw(
            GeometryVariants::D2(geometry.into()),
            color.into(),
            self.transform,
            None,
        );
    }

    fn draw_with_color_and_transform<C, TX>(&mut self, geometry: G, color: C, transform: TX)
    where
        C: Into<Color>,
        TX: Into<mint::ColumnMatrix4<f32>>,
    {
        self.push_draw(
            GeometryVariants::D2(geometry.into()),
            color.into(),
            transform.into(),
            None,
        );
    }

    fn image<T: Texture>(&mut self, geometry: G, texture: T) {
        self.push_draw(
            GeometryVariants::D2(geometry.into()),
            self.color,
            self.transform,
            Some(texture.into()),
        );
    }

    fn image_with_color<T, C>(&mut self, geometry: G, texture: T, color: C)
    where
        T: Texture,
        C: Into<Color>,
    {
        self.push_draw(
            GeometryVariants::D2(geometry.into()),
            color.into(),
            self.transform,
            Some(texture.into()),
        );
    }

    fn image_with_transform<T, TX>(&mut self, geometry: G, texture: T, transform: TX)
    where
        T: Texture,
        TX: Into<mint::ColumnMatrix4<f32>>,
    {
        self.push_draw(
            GeometryVariants::D2(geometry.into()),
            self.color,
            transform.into(),
            Some(texture.into()),
        );
    }

    fn image_with_color_and_transform<T, C, TX>(
        &mut self,
        geometry: G,
        texture: T,
        color: C,
        transform: TX,
    ) where
        T: Texture,
        C: Into<Color>,
        TX: Into<mint::ColumnMatrix4<f32>>,
    {
        self.push_draw(
            GeometryVariants::D2(geometry.into()),
            color.into(),
            transform.into(),
            Some(texture.into()),
        );
    }
}
impl<'a, G> crate::Stroke<crate::Vertex2D, G> for DrawList<'a>
where
    G: Into<Geometry<'a, crate::Vertex2D>>,
{
    fn stroke(&mut self, geometry: G) {
        self.stroke_with_color_and_transform(geometry, self.color, self.transform)
    }

    fn stroke_with_transform<TX>(&mut self, geometry: G, transform: TX)
    where
        TX: Into<mint::ColumnMatrix4<f32>>,
    {
        self.stroke_with_color_and_transform(geometry, self.color, transform)
    }

    fn stroke_with_color<C: Into<Color>>(&mut self, geometry: G, color: C) {
        self.stroke_with_color_and_transform(geometry, color, self.transform)
    }

    fn stroke_with_color_and_transform<C, TX>(&mut self, geometry: G, color: C, transform: TX)
    where
        C: Into<Color>,
        TX: Into<mint::ColumnMatrix4<f32>>,
    {
        let crate::Geometry { vertices, .. } = geometry.into();
        self.commands.push(Command::Line(DrawState {
            data: LineState {
                geometry: vertices
                    .iter()
                    .map(|v: &Vertex2D| LineVertex {
                        position: [v.position[0], v.position[1], 0.],
                        width: self.line_width,
                        color: [1., 1., 1., 1.],
                    })
                    .collect::<Vec<_>>()
                    .into(),
                is_loop: true,
                depth_buffer: false,
            },
            transform: transform.into(),
            camera: self.camera,
            projection_mode: self
                .projection_mode
                .unwrap_or(Projection::Orthographic(None)),
            color: color.into(),
            texture: None,
            target: self.target.clone(),
            shader: self.shader.clone(),
        }))
    }
}

pub trait SimpleConvexGeometry: std::fmt::Debug {
    type Vertices: Iterator<Item = Vertex2D>;
    fn vertices(&self) -> Self::Vertices;
    fn vertex_count(&self) -> usize;
}

macro_rules! impl_array_simple_convex_geom {
    ($ty:ty, $count:expr) => {
        impl SimpleConvexGeometry for [$ty; $count] {
            type Vertices = std::iter::Map<std::array::IntoIter<$ty, $count>, fn($ty) -> Vertex2D>;

            fn vertices(&self) -> Self::Vertices {
                IntoIterator::into_iter(*self).map(Into::into)
            }

            fn vertex_count(&self) -> usize {
                self.len()
            }
        }

        impl SimpleConvexGeometry for &[$ty; $count] {
            type Vertices = std::iter::Map<std::array::IntoIter<$ty, $count>, fn($ty) -> Vertex2D>;

            fn vertices(&self) -> Self::Vertices {
                IntoIterator::into_iter(**self).map(Into::into)
            }

            fn vertex_count(&self) -> usize {
                self.len()
            }
        }
    };
}

macro_rules! impl_32_array_simple_convex_geom {
    ($ty:ty) => {
        impl_array_simple_convex_geom!($ty, 1);
        impl_array_simple_convex_geom!($ty, 2);
        impl_array_simple_convex_geom!($ty, 3);
        impl_array_simple_convex_geom!($ty, 4);
        impl_array_simple_convex_geom!($ty, 5);
        impl_array_simple_convex_geom!($ty, 6);
        impl_array_simple_convex_geom!($ty, 7);
        impl_array_simple_convex_geom!($ty, 8);
        impl_array_simple_convex_geom!($ty, 9);
        impl_array_simple_convex_geom!($ty, 10);
        impl_array_simple_convex_geom!($ty, 11);
        impl_array_simple_convex_geom!($ty, 12);
        impl_array_simple_convex_geom!($ty, 13);
        impl_array_simple_convex_geom!($ty, 14);
        impl_array_simple_convex_geom!($ty, 15);
        impl_array_simple_convex_geom!($ty, 16);
        impl_array_simple_convex_geom!($ty, 17);
        impl_array_simple_convex_geom!($ty, 18);
        impl_array_simple_convex_geom!($ty, 19);
        impl_array_simple_convex_geom!($ty, 20);
        impl_array_simple_convex_geom!($ty, 21);
        impl_array_simple_convex_geom!($ty, 22);
        impl_array_simple_convex_geom!($ty, 23);
        impl_array_simple_convex_geom!($ty, 24);
        impl_array_simple_convex_geom!($ty, 25);
        impl_array_simple_convex_geom!($ty, 26);
        impl_array_simple_convex_geom!($ty, 27);
        impl_array_simple_convex_geom!($ty, 28);
        impl_array_simple_convex_geom!($ty, 29);
        impl_array_simple_convex_geom!($ty, 30);
        impl_array_simple_convex_geom!($ty, 31);
        impl_array_simple_convex_geom!($ty, 32);
    };
}

impl_32_array_simple_convex_geom!((f32, f32));
impl_32_array_simple_convex_geom!((f64, f64));
impl_32_array_simple_convex_geom!(Point);

impl<'a> SimpleConvexGeometry for &'a [Vertex2D] {
    type Vertices = std::iter::Copied<std::slice::Iter<'a, Vertex2D>>;

    fn vertices(&self) -> Self::Vertices {
        self.into_iter().copied()
    }

    fn vertex_count(&self) -> usize {
        self.len()
    }
}

impl<'a> SimpleConvexGeometry for &'a [Point] {
    type Vertices = std::iter::Map<std::slice::Iter<'a, Point>, fn(&'a Point) -> Vertex2D>;

    fn vertices(&self) -> Self::Vertices {
        self.iter().map(|p| (*p).into())
    }

    fn vertex_count(&self) -> usize {
        self.len()
    }
}

impl<'a> SimpleConvexGeometry for Vec<Point> {
    type Vertices = std::iter::Map<std::vec::IntoIter<Point>, fn(Point) -> Vertex2D>;

    fn vertices(&self) -> Self::Vertices {
        self.clone().into_iter().map(Into::into)
    }

    fn vertex_count(&self) -> usize {
        self.len()
    }
}

impl<'a> SimpleConvexGeometry for &'a [(f32, f32)] {
    type Vertices =
        std::iter::Map<std::slice::Iter<'a, (f32, f32)>, fn(&'a (f32, f32)) -> Vertex2D>;

    fn vertices(&self) -> Self::Vertices {
        self.iter().map(|p| (*p).into())
    }

    fn vertex_count(&self) -> usize {
        self.len()
    }
}

impl<'a> SimpleConvexGeometry for &'a [(f64, f64)] {
    type Vertices =
        std::iter::Map<std::slice::Iter<'a, (f64, f64)>, fn(&'a (f64, f64)) -> Vertex2D>;

    fn vertices(&self) -> Self::Vertices {
        self.iter().map(|p| (*p).into())
    }

    fn vertex_count(&self) -> usize {
        self.len()
    }
}
