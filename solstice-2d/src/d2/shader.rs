use solstice::shader::{Attribute, DynamicShader, Uniform, UniformLocation};
use solstice::{Context, ShaderKey};

#[derive(Debug)]
pub enum Shader2DError {
    GraphicsError(solstice::GraphicsError),
    UniformNotFound(String),
}

#[derive(Eq, PartialEq)]
struct TextureCache {
    ty: solstice::texture::TextureType,
    key: solstice::TextureKey,
    unit: solstice::TextureUnit,
}

#[allow(unused)]
pub struct Shader2D {
    inner: solstice::shader::DynamicShader,

    projection_location: UniformLocation,
    projection_cache: mint::ColumnMatrix4<f32>,
    view_location: UniformLocation,
    view_cache: mint::ColumnMatrix4<f32>,
    model_location: UniformLocation,
    model_cache: mint::ColumnMatrix4<f32>,
    color_location: UniformLocation,
    color_cache: mint::Vector4<f32>,
    tex0_location: UniformLocation,
    tex0_cache: TextureCache,
}

const SHADER_SRC: &str = include_str!("shader.glsl");

fn ortho(width: f32, height: f32) -> [[f32; 4]; 4] {
    let left = 0.;
    let right = width;
    let bottom = height;
    let top = 0.;
    let near = 0.;
    let far = 1000.;

    let c0r0 = 2. / (right - left);
    let c0r1 = 0.;
    let c0r2 = 0.;
    let c0r3 = 0.;

    let c1r0 = 0.;
    let c1r1 = 2. / (top - bottom);
    let c1r2 = 0.;
    let c1r3 = 0.;

    let c2r0 = 0.;
    let c2r1 = 0.;
    let c2r2 = -2. / (far - near);
    let c2r3 = 0.;

    let c3r0 = -(right + left) / (right - left);
    let c3r1 = -(top + bottom) / (top - bottom);
    let c3r2 = -(far + near) / (far - near);
    let c3r3 = 1.;

    #[cfg_attr(rustfmt, rustfmt_skip)]
    [
        [c0r0, c0r1, c0r2, c0r3],
        [c1r0, c1r1, c1r2, c1r3],
        [c2r0, c2r1, c2r2, c2r3],
        [c3r0, c3r1, c3r2, c3r3],
    ]
}

fn get_location(
    shader: &solstice::shader::DynamicShader,
    name: &str,
) -> Result<UniformLocation, Shader2DError> {
    shader
        .get_uniform_by_name(name)
        .ok_or(Shader2DError::UniformNotFound(name.to_owned()))
        .map(|uniform| uniform.location.clone())
}

impl Shader2D {
    pub fn new(ctx: &mut Context, width: f32, height: f32) -> Result<Self, Shader2DError> {
        let (vertex, fragment) =
            solstice::shader::DynamicShader::create_source(SHADER_SRC, SHADER_SRC);
        let shader = DynamicShader::new(ctx, vertex.as_str(), fragment.as_str())
            .map_err(Shader2DError::GraphicsError)?;

        let projection_location = get_location(&shader, "uProjection")?;
        let view_location = get_location(&shader, "uView")?;
        let model_location = get_location(&shader, "uModel")?;
        let color_location = get_location(&shader, "uColor")?;
        let tex0_location = get_location(&shader, "tex0")?;

        let projection_cache = ortho(width, height).into();
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let identity: mint::ColumnMatrix4<f32> = [
            1., 0., 0., 0.,
            0., 1., 0., 0.,
            0., 0., 1., 0.,
            0., 0., 0., 1.,
        ].into();
        let white: mint::Vector4<f32> = [1., 1., 1., 1.].into();

        ctx.use_shader(Some(&shader));
        ctx.set_uniform_by_location(
            &projection_location,
            &solstice::shader::RawUniformValue::Mat4(projection_cache),
        );
        ctx.set_uniform_by_location(
            &view_location,
            &solstice::shader::RawUniformValue::Mat4(identity),
        );
        ctx.set_uniform_by_location(
            &model_location,
            &solstice::shader::RawUniformValue::Mat4(identity),
        );
        ctx.set_uniform_by_location(
            &color_location,
            &solstice::shader::RawUniformValue::Vec4(white),
        );
        ctx.set_uniform_by_location(
            &tex0_location,
            &solstice::shader::RawUniformValue::SignedInt(0),
        );

        Ok(Self {
            inner: shader,
            projection_location,
            projection_cache,
            view_location,
            view_cache: identity,
            model_location,
            model_cache: identity,
            color_location,
            color_cache: white,
            tex0_location,
            tex0_cache: TextureCache {
                ty: solstice::texture::TextureType::Tex2D,
                key: Default::default(),
                unit: 0.into(),
            },
        })
    }

    pub fn set_width_height(&mut self, width: f32, height: f32) {
        let projection_cache = ortho(width, height).into();
        self.projection_cache = projection_cache;
    }

    pub fn bind_texture<T: solstice::texture::Texture>(&mut self, texture: T) {
        self.tex0_cache = TextureCache {
            ty: texture.get_texture_type(),
            key: texture.get_texture_key(),
            unit: 0.into(),
        };
    }

    pub fn is_bound<T: solstice::texture::Texture>(&self, texture: T) -> bool {
        let tex0 = TextureCache {
            ty: texture.get_texture_type(),
            key: texture.get_texture_key(),
            unit: 0.into(),
        };
        self.tex0_cache == tex0
    }

    pub fn activate(&self, ctx: &mut Context) -> &solstice::shader::DynamicShader {
        ctx.use_shader(Some(&self.inner));
        ctx.bind_texture_to_unit(
            self.tex0_cache.ty,
            self.tex0_cache.key,
            self.tex0_cache.unit,
        );
        ctx.set_uniform_by_location(
            &self.projection_location,
            &solstice::shader::RawUniformValue::Mat4(self.projection_cache),
        );
        &self.inner
    }
}

impl solstice::shader::Shader for Shader2D {
    fn handle(&self) -> ShaderKey {
        self.inner.handle()
    }

    fn attributes(&self) -> &[Attribute] {
        self.inner.attributes()
    }

    fn uniforms(&self) -> &[Uniform] {
        self.inner.uniforms()
    }
}
