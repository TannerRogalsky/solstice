mod shader;
mod shapes;
mod transform;

use crate::{BoxedGeometry, Color, Geometry};
pub use shader::Shader3D;
pub use shapes::*;
use solstice::mesh::MappedIndexedMesh;
use solstice::texture::Texture;
use solstice::Context;
pub use transform::*;

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

#[derive(PartialEq, Clone, Debug)]
struct TextureCache {
    ty: solstice::texture::TextureType,
    key: solstice::TextureKey,
    info: solstice::texture::TextureInfo,
}

impl solstice::texture::Texture for &TextureCache {
    fn get_texture_key(&self) -> solstice::TextureKey {
        self.key
    }

    fn get_texture_type(&self) -> solstice::texture::TextureType {
        self.ty
    }

    fn get_texture_info(&self) -> solstice::texture::TextureInfo {
        self.info
    }
}

#[derive(Clone, Debug)]
enum GeometryVariants {
    D2(std::boxed::Box<dyn BoxedGeometry<'static, crate::Vertex2D, u32>>),
    D3(std::boxed::Box<dyn BoxedGeometry<'static, Vertex3D, u32>>),
}

#[derive(Clone, Debug)]
pub struct DrawState {
    geometry: GeometryVariants,
    transform: Transform,
    color: Color,
    texture: Option<TextureCache>,
    target: Option<crate::Canvas>,
}

#[derive(Clone, Debug)]
pub enum Command {
    Draw(DrawState),
    Clear(Color, Option<crate::Canvas>),
}

#[derive(Clone, Debug, Default)]
pub struct DrawList {
    commands: Vec<Command>,
    color: Color,
    target: Option<crate::Canvas>,
}

impl DrawList {
    pub fn clear<C: Into<Color>>(&mut self, color: C) {
        self.commands
            .push(Command::Clear(color.into(), self.target.clone()))
    }

    pub fn set_color<C: Into<Color>>(&mut self, color: C) {
        self.color = color.into();
    }

    pub fn set_canvas(&mut self, target: Option<crate::Canvas>) {
        self.target = target;
    }
}

pub trait Draw<V: solstice::vertex::Vertex, G: Geometry<V> + Clone + 'static> {
    fn draw(&mut self, geometry: G);
    fn draw_with_transform<TX: Into<Transform>>(&mut self, geometry: G, transform: TX);
    fn draw_with_color<C: Into<Color>>(&mut self, geometry: G, color: C);
    fn draw_with_color_and_transform<C: Into<Color>, TX: Into<Transform>>(
        &mut self,
        geometry: G,
        color: C,
        transform: TX,
    );
    fn image<T: Texture>(&mut self, geometry: G, texture: T);
    fn image_with_color<T, C>(&mut self, geometry: G, texture: T, color: C)
    where
        T: Texture,
        C: Into<Color>;
    fn image_with_transform<T, TX>(&mut self, geometry: G, texture: T, transform: TX)
    where
        T: Texture,
        TX: Into<Transform>;
    fn image_with_color_and_transform<T, C, TX>(
        &mut self,
        geometry: G,
        texture: T,
        color: C,
        transform: TX,
    ) where
        T: Texture,
        C: Into<Color>,
        TX: Into<Transform>;
}

impl<G> Draw<Vertex3D, G> for DrawList
where
    G: Geometry<Vertex3D> + Clone + 'static,
{
    fn draw(&mut self, geometry: G) {
        self.draw_with_color_and_transform(geometry, self.color, Transform::default())
    }

    fn draw_with_transform<TX: Into<Transform>>(&mut self, geometry: G, transform: TX) {
        self.draw_with_color_and_transform(geometry, self.color, transform)
    }

    fn draw_with_color<C: Into<Color>>(&mut self, geometry: G, color: C) {
        self.draw_with_color_and_transform(geometry, color, Transform::default())
    }

    fn draw_with_color_and_transform<C: Into<Color>, TX: Into<Transform>>(
        &mut self,
        geometry: G,
        color: C,
        transform: TX,
    ) {
        self.commands.push(Command::Draw(DrawState {
            geometry: GeometryVariants::D3(std::boxed::Box::new(geometry)),
            transform: transform.into(),
            color: color.into(),
            texture: None,
            target: self.target.clone(),
        }))
    }

    fn image<T: Texture>(&mut self, geometry: G, texture: T) {
        self.image_with_color_and_transform(geometry, texture, self.color, Transform::default())
    }

    fn image_with_color<T, C>(&mut self, geometry: G, texture: T, color: C)
    where
        T: Texture,
        C: Into<Color>,
    {
        self.image_with_color_and_transform(geometry, texture, color, Transform::default())
    }

    fn image_with_transform<T, TX>(&mut self, geometry: G, texture: T, transform: TX)
    where
        T: Texture,
        TX: Into<Transform>,
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
        TX: Into<Transform>,
    {
        self.commands.push(Command::Draw(DrawState {
            geometry: GeometryVariants::D3(std::boxed::Box::new(geometry)),
            transform: transform.into(),
            color: color.into(),
            texture: Some(TextureCache {
                ty: texture.get_texture_type(),
                key: texture.get_texture_key(),
                info: texture.get_texture_info(),
            }),
            target: self.target.clone(),
        }))
    }
}

impl<G> Draw<crate::Vertex2D, G> for DrawList
where
    G: Geometry<crate::Vertex2D> + Clone + 'static,
{
    fn draw(&mut self, geometry: G) {
        self.draw_with_color_and_transform(geometry, self.color, Transform::default())
    }

    fn draw_with_transform<TX: Into<Transform>>(&mut self, geometry: G, transform: TX) {
        self.draw_with_color_and_transform(geometry, self.color, transform)
    }

    fn draw_with_color<C: Into<Color>>(&mut self, geometry: G, color: C) {
        self.draw_with_color_and_transform(geometry, color, Transform::default())
    }

    fn draw_with_color_and_transform<C: Into<Color>, TX: Into<Transform>>(
        &mut self,
        geometry: G,
        color: C,
        transform: TX,
    ) {
        self.commands.push(Command::Draw(DrawState {
            geometry: GeometryVariants::D2(std::boxed::Box::new(geometry)),
            transform: transform.into(),
            color: color.into(),
            texture: None,
            target: self.target.clone(),
        }))
    }

    fn image<T: Texture>(&mut self, geometry: G, texture: T) {
        self.image_with_color_and_transform(geometry, texture, self.color, Transform::default())
    }

    fn image_with_color<T, C>(&mut self, geometry: G, texture: T, color: C)
    where
        T: Texture,
        C: Into<Color>,
    {
        self.image_with_color_and_transform(geometry, texture, color, Transform::default())
    }

    fn image_with_transform<T, TX>(&mut self, geometry: G, texture: T, transform: TX)
    where
        T: Texture,
        TX: Into<Transform>,
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
        TX: Into<Transform>,
    {
        self.commands.push(Command::Draw(DrawState {
            geometry: GeometryVariants::D2(std::boxed::Box::new(geometry)),
            transform: transform.into(),
            color: color.into(),
            texture: Some(TextureCache {
                ty: texture.get_texture_type(),
                key: texture.get_texture_key(),
                info: texture.get_texture_info(),
            }),
            target: self.target.clone(),
        }))
    }
}

pub struct Graphics3D {
    mesh3d: MappedIndexedMesh<Vertex3D, u32>,
    mesh2d: MappedIndexedMesh<super::Vertex2D, u32>,
    default_shader3d: Shader3D,
    default_shader2d: super::Shader2D,
    default_texture: solstice::image::Image,
    width: f32,
    height: f32,
}

impl Graphics3D {
    pub fn new(ctx: &mut Context, width: f32, height: f32) -> Result<Self, crate::Graphics2DError> {
        let mesh2d = MappedIndexedMesh::new(ctx, 10000, 10000)?;
        let mesh3d = MappedIndexedMesh::new(ctx, 10000, 10000)?;
        // FIXME: NO UNWRAP
        let default_shader3d = Shader3D::new(ctx).unwrap();
        let default_shader2d = super::Shader2D::new(ctx)?;
        let default_texture = crate::create_default_texture(ctx)?;

        Ok(Self {
            mesh3d,
            mesh2d,
            default_shader3d,
            default_shader2d,
            default_texture,
            width,
            height,
        })
    }

    pub fn process(&mut self, ctx: &mut Context, draw_list: &mut DrawList) {
        for command in draw_list.commands.drain(..) {
            match command {
                Command::Draw(draw_state) => {
                    let DrawState {
                        geometry,
                        transform,
                        color,
                        texture,
                        target,
                    } = draw_state;
                    match geometry {
                        GeometryVariants::D2(geometry) => {
                            let transform_verts = |mut v: crate::Vertex2D| -> crate::Vertex2D {
                                v.color = color.into();
                                v
                            };

                            let vertices = geometry
                                .vertices()
                                .map(transform_verts)
                                .collect::<std::boxed::Box<[_]>>();
                            let indices = geometry.indices().collect::<std::boxed::Box<[_]>>();
                            self.mesh2d.set_vertices(&vertices, 0);
                            self.mesh2d.set_indices(&indices, 0);
                            let mesh = self.mesh2d.unmap(ctx);
                            let geometry = solstice::Geometry {
                                mesh,
                                draw_range: 0..indices.len(),
                                draw_mode: solstice::DrawMode::Triangles,
                                instance_count: 1,
                            };
                            self.default_shader2d
                                .set_width_height(self.width, self.height, false);
                            self.default_shader2d.send_uniform(
                                "uModel",
                                solstice::shader::RawUniformValue::Mat4(transform.inner.into()),
                            );
                            match texture.as_ref() {
                                None => self.default_shader2d.bind_texture(&self.default_texture),
                                Some(texture) => self.default_shader2d.bind_texture(texture),
                            }
                            self.default_shader2d.activate(ctx);
                            solstice::Renderer::draw(
                                ctx,
                                &self.default_shader2d,
                                &geometry,
                                solstice::PipelineSettings {
                                    depth_state: None,
                                    framebuffer: target.as_ref().map(|c| &c.inner),
                                    ..solstice::PipelineSettings::default()
                                },
                            );
                        }
                        GeometryVariants::D3(geometry) => {
                            let vertices = geometry.vertices().collect::<std::boxed::Box<[_]>>();
                            let indices = geometry.indices().collect::<std::boxed::Box<[_]>>();
                            self.mesh3d.set_vertices(&vertices, 0);
                            self.mesh3d.set_indices(&indices, 0);
                            let mesh = self.mesh3d.unmap(ctx);
                            let geometry = solstice::Geometry {
                                mesh,
                                draw_range: 0..indices.len(),
                                draw_mode: solstice::DrawMode::Triangles,
                                instance_count: 1,
                            };
                            self.default_shader3d
                                .set_width_height(self.width, self.height, false);
                            self.default_shader3d.send_uniform(
                                "uModel",
                                solstice::shader::RawUniformValue::Mat4(transform.inner.into()),
                            );
                            self.default_shader3d.set_color(draw_state.color);
                            match texture.as_ref() {
                                None => self.default_shader3d.bind_texture(&self.default_texture),
                                Some(texture) => self.default_shader3d.bind_texture(texture),
                            }
                            self.default_shader3d.activate(ctx);
                            solstice::Renderer::draw(
                                ctx,
                                &self.default_shader3d,
                                &geometry,
                                solstice::PipelineSettings {
                                    framebuffer: target.as_ref().map(|c| &c.inner),
                                    ..solstice::PipelineSettings::default()
                                },
                            );
                        }
                    };
                }
                Command::Clear(color, target) => {
                    solstice::Renderer::clear(
                        ctx,
                        solstice::ClearSettings {
                            color: Some(color.into()),
                            target: target.as_ref().map(|c| &c.inner),
                            ..solstice::ClearSettings::default()
                        },
                    );
                }
            }
        }
    }
}
