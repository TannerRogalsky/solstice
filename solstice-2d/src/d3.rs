mod shapes;
mod transform;

pub use shapes::*;
pub use transform::*;

use super::{
    BoxedGeometry, Color, Command, Draw, DrawList, DrawState, Geometry, GeometryVariants,
    LineState, LineVertex, Projection, TextureCache,
};
use solstice::texture::Texture;

#[derive(Debug, PartialEq, Copy, Clone, solstice::vertex::Vertex)]
#[repr(C)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub normal: [f32; 3],
}

impl<'a, V, I, G> BoxedGeometry<'a, Vertex3D, u32> for G
where
    V: Iterator<Item = Vertex3D> + 'a,
    I: Iterator<Item = u32> + 'a,
    G: Geometry<Vertex3D, Vertices = V, Indices = I> + dyn_clone::DynClone + std::fmt::Debug,
{
    fn vertices(&self) -> std::boxed::Box<dyn Iterator<Item = Vertex3D> + 'a> {
        std::boxed::Box::new(Geometry::vertices(self))
    }

    fn indices(&self) -> std::boxed::Box<dyn Iterator<Item = u32> + 'a> {
        std::boxed::Box::new(Geometry::indices(self))
    }
}
dyn_clone::clone_trait_object!(BoxedGeometry<'_, Vertex3D, u32>);

impl<G> Draw<Vertex3D, G> for DrawList
where
    G: Geometry<Vertex3D> + Clone + 'static,
{
    fn draw(&mut self, geometry: G) {
        self.draw_with_color_and_transform(geometry, self.color, self.transform)
    }

    fn draw_with_transform<TX: Into<Transform3D>>(&mut self, geometry: G, transform: TX) {
        self.draw_with_color_and_transform(geometry, self.color, transform)
    }

    fn draw_with_color<C: Into<Color>>(&mut self, geometry: G, color: C) {
        self.draw_with_color_and_transform(geometry, color, self.transform)
    }

    fn draw_with_color_and_transform<C: Into<Color>, TX: Into<Transform3D>>(
        &mut self,
        geometry: G,
        color: C,
        transform: TX,
    ) {
        self.commands.push(Command::Draw(DrawState {
            data: GeometryVariants::D3(std::boxed::Box::new(geometry)),
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

    fn stroke(&mut self, geometry: G) {
        self.stroke_with_color_and_transform(geometry, self.color, self.transform)
    }

    fn stroke_with_transform<TX: Into<Transform3D>>(&mut self, geometry: G, transform: TX) {
        self.stroke_with_color_and_transform(geometry, self.color, transform)
    }

    fn stroke_with_color<C: Into<Color>>(&mut self, geometry: G, color: C) {
        self.stroke_with_color_and_transform(geometry, color, self.transform)
    }

    fn stroke_with_color_and_transform<C: Into<Color>, TX: Into<Transform3D>>(
        &mut self,
        geometry: G,
        color: C,
        transform: TX,
    ) {
        self.commands.push(Command::Line(DrawState {
            data: LineState {
                geometry: std::boxed::Box::new(
                    geometry
                        .vertices()
                        .map(|v: Vertex3D| LineVertex {
                            position: v.position,
                            width: 10.0,
                            color: [1., 1., 1., 1.],
                        })
                        .collect::<Vec<_>>()
                        .into_iter(),
                ),
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

    fn image<T: Texture>(&mut self, geometry: G, texture: T) {
        self.image_with_color_and_transform(geometry, texture, self.color, self.transform)
    }

    fn image_with_color<T, C>(&mut self, geometry: G, texture: T, color: C)
    where
        T: Texture,
        C: Into<Color>,
    {
        self.image_with_color_and_transform(geometry, texture, color, self.transform)
    }

    fn image_with_transform<T, TX>(&mut self, geometry: G, texture: T, transform: TX)
    where
        T: Texture,
        TX: Into<Transform3D>,
    {
        self.image_with_color_and_transform(geometry, texture, self.color, transform)
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
        TX: Into<Transform3D>,
    {
        self.commands.push(Command::Draw(DrawState {
            data: GeometryVariants::D3(std::boxed::Box::new(geometry)),
            transform: transform.into(),
            camera: self.camera,
            projection_mode: self
                .projection_mode
                .unwrap_or(Projection::Perspective(None)),
            color: color.into(),
            texture: Some(TextureCache {
                ty: texture.get_texture_type(),
                key: texture.get_texture_key(),
                info: texture.get_texture_info(),
            }),
            target: self.target.clone(),
            shader: self.shader.clone(),
        }))
    }
}
