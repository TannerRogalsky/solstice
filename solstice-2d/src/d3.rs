mod shapes;
mod transform;

pub use shapes::*;
pub use transform::*;

use super::{
    Color, Command, Draw, DrawList, DrawState, Geometry, GeometryVariants, LineState, LineVertex,
    Projection,
};
use bytemuck::{Pod, Zeroable};
use solstice::texture::Texture;

#[repr(C)]
#[derive(Zeroable, Pod, Debug, PartialEq, Copy, Clone, solstice::vertex::Vertex)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub normal: [f32; 3],
}

impl From<Vec<Vertex3D>> for Geometry<'_, Vertex3D> {
    fn from(vertices: Vec<Vertex3D>) -> Self {
        Self::new::<_, Vec<u32>>(vertices, None)
    }
}

impl<'a> From<&'a [Vertex3D]> for Geometry<'a, Vertex3D> {
    fn from(vertices: &'a [Vertex3D]) -> Self {
        Self::new::<_, Vec<u32>>(vertices, None)
    }
}

impl<'a, G> Draw<Vertex3D, G> for DrawList<'a>
where
    G: crate::GeometryKind<'a, Vertex3D> + 'a,
{
    fn draw(&mut self, geometry: G) {
        self.push_draw(
            GeometryVariants::D3(geometry.into()),
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
            GeometryVariants::D3(geometry.into()),
            self.color,
            transform.into(),
            None,
        );
    }

    fn draw_with_color<C: Into<Color>>(&mut self, geometry: G, color: C) {
        self.push_draw(
            GeometryVariants::D3(geometry.into()),
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
            GeometryVariants::D3(geometry.into()),
            color.into(),
            transform.into(),
            None,
        );
    }

    fn image<T: Texture>(&mut self, geometry: G, texture: T) {
        self.push_draw(
            GeometryVariants::D3(geometry.into()),
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
            GeometryVariants::D3(geometry.into()),
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
            GeometryVariants::D3(geometry.into()),
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
            GeometryVariants::D3(geometry.into()),
            color.into(),
            transform.into(),
            Some(texture.into()),
        );
    }
}
impl<'a, G> crate::Stroke<crate::Vertex3D, G> for DrawList<'a>
where
    G: Into<Geometry<'a, crate::Vertex3D>>,
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
                    .map(|v: &Vertex3D| LineVertex {
                        position: v.position,
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
                .unwrap_or(Projection::Perspective(None)),
            color: color.into(),
            texture: None,
            target: self.target.clone(),
            shader: self.shader.clone(),
        }))
    }
}
