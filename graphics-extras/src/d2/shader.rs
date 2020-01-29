use graphics::shader::UniformLocation;
use graphics::Context;
use std::{cell::RefCell, rc::Rc};

pub enum Shader2DError {
    ShaderNotFound,
    GraphicsError(graphics::GraphicsError),
    UniformNotFound(String),
}

pub struct Shader2D {
    gfx: Rc<RefCell<Context>>,
    inner: graphics::ShaderKey,

    projection_location: UniformLocation,
    projection_cache: mint::ColumnMatrix4<f32>,
    view_location: UniformLocation,
    view_cache: mint::ColumnMatrix4<f32>,
    model_location: UniformLocation,
    model_cache: mint::ColumnMatrix4<f32>,
    color_location: UniformLocation,
    color_cache: mint::Vector4<f32>,
    tex0_location: UniformLocation,
    tex0_cache: i32,
}

const SHADER_SRC: &str = include_str!("shader.glsl");

fn ortho(width: f32, height: f32) -> [[f32; 4]; 4] {
    cgmath::ortho(0., width, height, 0., 0., 1000.).into()
}

fn get_location(
    gfx: &mut Context,
    inner: graphics::ShaderKey,
    name: &str,
) -> Result<UniformLocation, Shader2DError> {
    gfx.get_shader(inner)
        .ok_or(Shader2DError::ShaderNotFound)
        .and_then(|shader| {
            shader
                .get_uniform_by_name(name)
                .ok_or(Shader2DError::UniformNotFound(name.to_owned()))
                .map(|uniform| uniform.location.clone())
        })
}

impl Shader2D {
    pub fn new(gfx: Rc<RefCell<Context>>, width: f32, height: f32) -> Result<Self, Shader2DError> {
        let (vertex, fragment) = graphics::shader::Shader::create_source(SHADER_SRC, SHADER_SRC);
        let inner = gfx
            .borrow_mut()
            .new_shader(vertex.as_str(), fragment.as_str())
            .map_err(Shader2DError::GraphicsError)?;

        let projection_location = get_location(&mut gfx.borrow_mut(), inner, "uProjection")?;
        let view_location = get_location(&mut gfx.borrow_mut(), inner, "uView")?;
        let model_location = get_location(&mut gfx.borrow_mut(), inner, "uModel")?;
        let color_location = get_location(&mut gfx.borrow_mut(), inner, "uColor")?;
        let tex0_location = get_location(&mut gfx.borrow_mut(), inner, "tex0")?;

        let projection_cache = ortho(width, height).into();
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let identity: mint::ColumnMatrix4<f32> = [
            1., 0., 0., 0.,
            0., 1., 0., 0.,
            0., 0., 1., 0.,
            0., 0., 0., 1.,
        ].into();
        let white: mint::Vector4<f32> = [1., 1., 1., 1.].into();

        gfx.borrow_mut().use_shader(Some(inner));
        gfx.borrow_mut().set_uniform_by_location(
            &projection_location,
            &graphics::shader::RawUniformValue::Mat4(projection_cache),
        );
        gfx.borrow_mut().set_uniform_by_location(
            &view_location,
            &graphics::shader::RawUniformValue::Mat4(identity),
        );
        gfx.borrow_mut().set_uniform_by_location(
            &model_location,
            &graphics::shader::RawUniformValue::Mat4(identity),
        );
        gfx.borrow_mut().set_uniform_by_location(
            &color_location,
            &graphics::shader::RawUniformValue::Vec4(white),
        );
        gfx.borrow_mut().set_uniform_by_location(
            &tex0_location,
            &graphics::shader::RawUniformValue::SignedInt(0),
        );

        Ok(Self {
            gfx,
            inner,
            projection_location,
            projection_cache,
            view_location,
            view_cache: identity,
            model_location,
            model_cache: identity,
            color_location,
            color_cache: white,
            tex0_location,
            tex0_cache: 0,
        })
    }

    pub fn set_color(&mut self, color: mint::Vector4<f32>) {
        if color != self.color_cache {
            self.color_cache = color;
            self.gfx.borrow_mut().set_uniform_by_location(
                &self.color_location,
                &graphics::shader::RawUniformValue::Vec4(self.color_cache),
            )
        }
    }

    pub fn set_width_height(&mut self, width: f32, height: f32) {
        let projection_cache = ortho(width, height).into();
        if projection_cache != self.projection_cache {
            self.projection_cache = projection_cache;
            self.gfx.borrow_mut().set_uniform_by_location(
                &self.projection_location,
                &graphics::shader::RawUniformValue::Mat4(self.projection_cache),
            );
        }
    }

    pub fn bind_texture<T: graphics::texture::Texture>(&mut self, texture: T) {
        self.gfx.borrow_mut().bind_texture_to_unit(
            texture.get_texture_type(),
            texture.get_texture_key(),
            0.into(),
        );
    }
}
