use solstice::shader::{Attribute, DynamicShader, Uniform, UniformLocation};
use solstice::{Context, ShaderKey};

#[derive(Debug)]
pub enum Shader2DError {
    GraphicsError(solstice::GraphicsError),
    UniformNotFound(String),
}

impl std::fmt::Display for Shader2DError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Shader2DError {}

#[derive(Eq, PartialEq)]
struct TextureCache {
    ty: solstice::texture::TextureType,
    key: solstice::TextureKey,
    location: Option<UniformLocation>,
}

const MAX_TEXTURE_UNITS: usize = 8;

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

    textures: [TextureCache; MAX_TEXTURE_UNITS],

    other_uniforms: std::collections::HashMap<String, solstice::shader::RawUniformValue>,
}

const DEFAULT_VERT: &str = r#"
vec4 pos(mat4 transform_projection, vec4 vertex_position) {
    return transform_projection * vertex_position;
}
"#;

const DEFAULT_FRAG: &str = r#"
vec4 effect(vec4 color, Image texture, vec2 texture_coords, vec2 screen_coords) {
    return Texel(texture, texture_coords) * color;
}
"#;

fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
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
        .ok_or_else(|| Shader2DError::UniformNotFound(name.to_owned()))
        .map(|uniform| uniform.location.clone())
}

pub struct ShaderSource<'a> {
    vertex: &'a str,
    fragment: &'a str,
}

impl<'a> From<&'a str> for ShaderSource<'a> {
    fn from(src: &'a str) -> Self {
        Self {
            vertex: src,
            fragment: src,
        }
    }
}

impl<'a> From<(&'a str, &'a str)> for ShaderSource<'a> {
    fn from((vertex, fragment): (&'a str, &'a str)) -> Self {
        Self { vertex, fragment }
    }
}

fn shader_src(src: ShaderSource) -> String {
    format!(
        "#define Image sampler2D
#define ArrayImage sampler2DArray
#define CubeImage samplerCube
#define VolumeImage sampler3D

varying vec4 vColor;
varying vec2 vUV;

#ifdef VERTEX
attribute vec4 position;
attribute vec4 color;
attribute vec2 uv;

uniform mat4 uProjection;
uniform mat4 uView;
uniform mat4 uModel;

{vertex}

void main() {{
    vColor = color;
    vUV = uv;
    gl_Position = pos(uProjection * uView * uModel, position);
}}
#endif

#ifdef FRAGMENT
uniform sampler2D tex0;
uniform vec4 uColor;

{fragment}

void main() {{
    fragColor = effect(uColor * vColor, tex0, vUV, vUV);
}}
#endif",
        vertex = src.vertex,
        fragment = src.fragment
    )
}

impl Shader2D {
    pub fn new(ctx: &mut Context) -> Result<Self, Shader2DError> {
        Self::with((DEFAULT_VERT, DEFAULT_FRAG), ctx)
    }

    pub fn with<'a, S>(src: S, ctx: &mut Context) -> Result<Self, Shader2DError>
    where
        S: Into<ShaderSource<'a>>,
    {
        let src = shader_src(src.into());
        let (vertex, fragment) =
            solstice::shader::DynamicShader::create_source(src.as_str(), src.as_str());
        let shader = DynamicShader::new(ctx, vertex.as_str(), fragment.as_str())
            .map_err(Shader2DError::GraphicsError)?;

        let projection_location = get_location(&shader, "uProjection")?;
        let view_location = get_location(&shader, "uView")?;
        let model_location = get_location(&shader, "uModel")?;
        let color_location = get_location(&shader, "uColor")?;
        let mut textures = (0..MAX_TEXTURE_UNITS).map(|i| {
            let location = get_location(&shader, ("tex".to_owned() + &i.to_string()).as_str()).ok();
            TextureCache {
                ty: solstice::texture::TextureType::Tex2D,
                key: Default::default(),
                location,
            }
        });
        let textures = [
            textures.next().unwrap(),
            textures.next().unwrap(),
            textures.next().unwrap(),
            textures.next().unwrap(),
            textures.next().unwrap(),
            textures.next().unwrap(),
            textures.next().unwrap(),
            textures.next().unwrap(),
        ];

        #[rustfmt::skip]
        let identity: mint::ColumnMatrix4<f32> = [
            1., 0., 0., 0.,
            0., 1., 0., 0.,
            0., 0., 1., 0.,
            0., 0., 0., 1.,
        ].into();
        let white: mint::Vector4<f32> = [1., 1., 1., 1.].into();
        let projection_cache = identity;

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
            textures,
            other_uniforms: Default::default(),
        })
    }

    pub fn set_width_height(&mut self, width: f32, height: f32, invert_y: bool) {
        let projection_cache = if invert_y {
            ortho(0., width, 0., height, 0., 1000.)
        } else {
            ortho(0., width, height, 0., 0., 1000.)
        };
        self.projection_cache = projection_cache.into();
    }

    pub fn bind_texture<T: solstice::texture::Texture>(&mut self, texture: T) {
        self.textures[0].key = texture.get_texture_key();
        self.textures[0].ty = texture.get_texture_type();
    }

    pub fn bind_texture_at_location<T: solstice::texture::Texture>(
        &mut self,
        texture: T,
        location: usize,
    ) {
        let cache = &mut self.textures[location];
        cache.key = texture.get_texture_key();
        cache.ty = texture.get_texture_type();
    }

    pub fn is_bound<T: solstice::texture::Texture>(&self, texture: T) -> bool {
        self.textures[0].key == texture.get_texture_key()
    }

    pub fn is_dirty(&self) -> bool {
        true
    }

    pub fn send_uniform<S, V>(&mut self, name: S, value: V)
    where
        S: AsRef<str>,
        V: std::convert::TryInto<solstice::shader::RawUniformValue>,
    {
        if let Some(uniform) = self.inner.get_uniform_by_name(name.as_ref()) {
            if let Some(data) = value.try_into().ok() {
                self.other_uniforms.insert(uniform.name.clone(), data);
            }
        }
    }

    pub fn activate(&mut self, ctx: &mut Context) {
        use solstice::shader::RawUniformValue::{Mat4, SignedInt};
        ctx.use_shader(Some(&self.inner));
        for (index, texture) in self.textures.iter().enumerate() {
            if let Some(location) = &texture.location {
                ctx.bind_texture_to_unit(texture.ty, texture.key, index.into());
                ctx.set_uniform_by_location(location, &SignedInt(index as _));
            }
        }
        for (name, data) in self.other_uniforms.iter() {
            let uniform = self.inner.get_uniform_by_name(name.as_str());
            if let Some(uniform) = uniform {
                ctx.set_uniform_by_location(&uniform.location, data);
            }
        }
        ctx.set_uniform_by_location(&self.projection_location, &Mat4(self.projection_cache));
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
