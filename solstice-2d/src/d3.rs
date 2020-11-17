mod shader;
mod shapes;
mod transform;

use crate::{AnyGeometry, Color};
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

pub trait Geometry: std::fmt::Debug {
    type Vertices: Iterator<Item = Vertex3D>;
    type Indices: Iterator<Item = u32>;

    fn vertices(&self) -> Self::Vertices;
    fn indices(&self) -> Self::Indices;
}

impl<'a, V, I, G> AnyGeometry<'a, Vertex3D, u32> for G
where
    V: Iterator<Item = Vertex3D> + 'a,
    I: Iterator<Item = u32> + 'a,
    G: Geometry<Vertices = V, Indices = I> + dyn_clone::DynClone + std::fmt::Debug,
{
    fn vertices(&self) -> std::boxed::Box<dyn Iterator<Item = Vertex3D> + 'a> {
        std::boxed::Box::new(Geometry::vertices(self))
    }

    fn indices(&self) -> std::boxed::Box<dyn Iterator<Item = u32> + 'a> {
        std::boxed::Box::new(Geometry::indices(self))
    }
}
dyn_clone::clone_trait_object!(AnyGeometry<'_, Vertex3D, u32>);

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
pub struct DrawState {
    geometry: std::boxed::Box<dyn AnyGeometry<'static, Vertex3D, u32>>,
    transform: Transform,
    texture: Option<TextureCache>,
}

#[derive(Clone, Debug)]
pub enum Command {
    Draw(DrawState),
    Clear(Color),
}

#[derive(Clone, Debug, Default)]
pub struct DrawList {
    commands: Vec<Command>,
}

pub trait ConcreteGeometry: Geometry + Clone + 'static {}
impl<T> ConcreteGeometry for T where T: Geometry + Clone + 'static {}

impl DrawList {
    pub fn clear<C: Into<Color>>(&mut self, color: C) {
        self.commands.push(Command::Clear(color.into()))
    }

    pub fn draw<G: ConcreteGeometry>(&mut self, geometry: G) {
        self.draw_with_transform(geometry, Transform::default())
    }

    pub fn draw_with_transform<G: ConcreteGeometry>(&mut self, geometry: G, transform: Transform) {
        self.commands.push(Command::Draw(DrawState {
            geometry: std::boxed::Box::new(geometry),
            transform,
            texture: None,
        }))
    }

    pub fn image<G: ConcreteGeometry, T: Texture>(&mut self, geometry: G, texture: T) {
        self.image_with_transform(geometry, texture, Transform::default())
    }

    pub fn image_with_transform<G: ConcreteGeometry, T: Texture>(
        &mut self,
        geometry: G,
        texture: T,
        transform: Transform,
    ) {
        self.commands.push(Command::Draw(DrawState {
            geometry: std::boxed::Box::new(geometry),
            transform,
            texture: Some(TextureCache {
                ty: texture.get_texture_type(),
                key: texture.get_texture_key(),
                info: texture.get_texture_info(),
            }),
        }))
    }
}

pub struct Graphics3D {
    mesh: MappedIndexedMesh<Vertex3D, u32>,
    default_shader: Shader3D,
    default_texture: solstice::image::Image,
    width: f32,
    height: f32,
}

impl Graphics3D {
    pub fn new(ctx: &mut Context, width: f32, height: f32) -> Result<Self, crate::Graphics2DError> {
        let mesh = MappedIndexedMesh::new(ctx, 10000, 10000)?;
        let default_shader = Shader3D::new(ctx).unwrap();
        let default_texture = crate::create_default_texture(ctx)?;

        Ok(Self {
            mesh,
            default_shader,
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
                        texture,
                    } = draw_state;
                    let vertices = geometry.vertices().collect::<std::boxed::Box<[_]>>();
                    let indices = geometry.indices().collect::<std::boxed::Box<[_]>>();
                    self.mesh.set_vertices(&vertices, 0);
                    self.mesh.set_indices(&indices, 0);
                    let mesh = self.mesh.unmap(ctx);
                    let geometry = solstice::Geometry {
                        mesh,
                        draw_range: 0..indices.len(),
                        draw_mode: solstice::DrawMode::Triangles,
                        instance_count: 1,
                    };
                    self.default_shader
                        .set_width_height(self.width, self.height, false);
                    self.default_shader.send_uniform(
                        "uModel",
                        solstice::shader::RawUniformValue::Mat4(transform.inner.into()),
                    );
                    match texture.as_ref() {
                        None => self.default_shader.bind_texture(&self.default_texture),
                        Some(texture) => self.default_shader.bind_texture(texture),
                    }
                    self.default_shader.activate(ctx);
                    solstice::Renderer::draw(
                        ctx,
                        &self.default_shader,
                        &geometry,
                        solstice::PipelineSettings::default(),
                    );
                }
                Command::Clear(color) => {
                    solstice::Renderer::clear(
                        ctx,
                        solstice::ClearSettings {
                            color: Some(color.into()),
                            target: None,
                            ..solstice::ClearSettings::default()
                        },
                    );
                }
            }
        }
    }
}
