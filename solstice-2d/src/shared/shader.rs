#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Orthographic {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    pub near: f32,
    pub far: f32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Perspective {
    pub aspect: f32,
    pub fovy: f32,
    pub near: f32,
    pub far: f32,
}

/// A None value means we will use the default projections of the given type
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Projection {
    Orthographic(Option<Orthographic>),
    Perspective(Option<Perspective>),
}

#[derive(Debug)]
pub enum ShaderError {
    GraphicsError(solstice::GraphicsError),
    UniformNotFound(String),
}

impl std::fmt::Display for ShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ShaderError {}

pub struct ShaderSource<'a> {
    pub vertex: &'a str,
    pub fragment: &'a str,
}

impl<'a> From<&'a String> for ShaderSource<'a> {
    fn from(src: &'a String) -> Self {
        Self {
            vertex: src.as_str(),
            fragment: src.as_str(),
        }
    }
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

use solstice::shader::{Attribute, DynamicShader, Uniform, UniformLocation};
use solstice::{Context, ShaderKey};

#[derive(Eq, PartialEq, Clone, Debug)]
struct TextureCache {
    ty: solstice::texture::TextureType,
    key: solstice::TextureKey,
    location: Option<UniformLocation>,
}

const MAX_TEXTURE_UNITS: usize = 8;

#[derive(Debug, Clone, PartialEq)]
pub struct Shader {
    inner: solstice::shader::DynamicShader,

    projection_location: Option<UniformLocation>,
    projection_cache: mint::ColumnMatrix4<f32>,
    view_location: Option<UniformLocation>,
    view_cache: mint::ColumnMatrix4<f32>,
    model_location: Option<UniformLocation>,
    model_cache: mint::ColumnMatrix4<f32>,
    normal_matrix_location: Option<UniformLocation>,
    color_location: Option<UniformLocation>,
    color_cache: mint::Vector4<f32>,
    resolution_location: Option<UniformLocation>,
    resolution_cache: mint::Vector4<f32>,

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

fn get_location(
    shader: &solstice::shader::DynamicShader,
    name: &str,
) -> Result<UniformLocation, ShaderError> {
    shader
        .get_uniform_by_name(name)
        .ok_or_else(|| ShaderError::UniformNotFound(name.to_owned()))
        .map(|uniform| uniform.location.clone())
}

fn shader_src(src: ShaderSource) -> String {
    format!(
        "#define Image sampler2D
#define ArrayImage sampler2DArray
#define CubeImage samplerCube
#define VolumeImage sampler3D

varying vec4 vColor;
varying vec2 vUV;

uniform SOLSTICE_HIGHP_OR_MEDIUMP vec4 uResolution;

#ifdef VERTEX
attribute vec4 position;
attribute vec4 color;
attribute vec3 normal;
attribute vec2 uv;

uniform mat4 uProjection;
uniform mat4 uView;
uniform mat4 uModel;
uniform mat4 uNormalMatrix;

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
    vec2 screen = vec2(gl_FragCoord.x, (gl_FragCoord.y * uResolution.z) + uResolution.w);
    fragColor = effect(uColor * vColor, tex0, vUV, screen);
}}
#endif",
        vertex = src.vertex,
        fragment = src.fragment
    )
}

impl Shader {
    pub fn new(ctx: &mut Context) -> Result<Self, ShaderError> {
        Self::with((DEFAULT_VERT, DEFAULT_FRAG), ctx)
    }

    pub fn with<'a, S>(src: S, ctx: &mut Context) -> Result<Self, ShaderError>
    where
        S: Into<ShaderSource<'a>>,
    {
        let src = shader_src(src.into());
        let (vertex, fragment) =
            solstice::shader::DynamicShader::create_source(src.as_str(), src.as_str());
        let shader = DynamicShader::new(ctx, vertex.as_str(), fragment.as_str())
            .map_err(ShaderError::GraphicsError)?;

        let projection_location = get_location(&shader, "uProjection").ok();
        let view_location = get_location(&shader, "uView").ok();
        let model_location = get_location(&shader, "uModel").ok();
        let normal_matrix_location = get_location(&shader, "uNormalMatrix").ok();
        let color_location = get_location(&shader, "uColor").ok();
        let resolution_location = get_location(&shader, "uResolution").ok();
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
        if let Some(projection_location) = &projection_location {
            ctx.set_uniform_by_location(
                &projection_location,
                &solstice::shader::RawUniformValue::Mat4(projection_cache),
            );
        }
        if let Some(view_location) = &view_location {
            ctx.set_uniform_by_location(
                &view_location,
                &solstice::shader::RawUniformValue::Mat4(identity),
            );
        }
        if let Some(model_location) = &model_location {
            ctx.set_uniform_by_location(
                &model_location,
                &solstice::shader::RawUniformValue::Mat4(identity),
            );
        }
        if let Some(normal_location) = &normal_matrix_location {
            ctx.set_uniform_by_location(
                &normal_location,
                &solstice::shader::RawUniformValue::Mat4(identity),
            );
        }
        if let Some(color_location) = &color_location {
            ctx.set_uniform_by_location(
                color_location,
                &solstice::shader::RawUniformValue::Vec4(white),
            );
        }

        Ok(Self {
            inner: shader,
            projection_location,
            projection_cache,
            view_location,
            view_cache: identity,
            model_location,
            model_cache: identity,
            normal_matrix_location,
            color_location,
            color_cache: white,
            resolution_location,
            resolution_cache: mint::Vector4 {
                x: 0f32,
                y: 0.,
                z: 0.,
                w: 0.,
            },
            textures,
            other_uniforms: Default::default(),
        })
    }

    pub fn set_viewport(
        &mut self,
        projection: Projection,
        default_projection_bounds: Option<crate::Rectangle>,
        viewport: solstice::viewport::Viewport<i32>,
        invert_y: bool,
    ) {
        let viewport = default_projection_bounds.unwrap_or_else(|| {
            crate::Rectangle::new(
                viewport.x() as _,
                viewport.y() as _,
                viewport.width() as _,
                viewport.height() as _,
            )
        });
        const FAR_PLANE: f32 = 1000.0;
        let projection_cache: mint::ColumnMatrix4<f32> = match projection {
            Projection::Orthographic(projection) => {
                let (top, bottom) = if invert_y {
                    (viewport.y + viewport.height, -viewport.y)
                } else {
                    (-viewport.y, viewport.y + viewport.height)
                };
                let Orthographic {
                    left,
                    right,
                    top,
                    bottom,
                    near,
                    far,
                } = projection.unwrap_or(Orthographic {
                    left: viewport.x,
                    right: viewport.x + viewport.width,
                    top,
                    bottom,
                    near: 0.0,
                    far: FAR_PLANE,
                });
                ortho(left, right, bottom, top, near, far).into()
            }
            Projection::Perspective(projection) => {
                let fovy = if invert_y {
                    -std::f32::consts::FRAC_PI_2
                } else {
                    std::f32::consts::FRAC_PI_2
                };
                let Perspective {
                    aspect,
                    fovy,
                    near,
                    far,
                } = projection.unwrap_or(Perspective {
                    aspect: viewport.width / viewport.height,
                    fovy,
                    near: 0.1,
                    far: FAR_PLANE,
                });
                nalgebra::Matrix4::new_perspective(aspect, fovy, near, far).into()
            }
        };

        self.resolution_cache.x = viewport.width;
        self.resolution_cache.y = viewport.height;
        if invert_y {
            self.resolution_cache.z = 1.;
            self.resolution_cache.w = 0.;
        } else {
            self.resolution_cache.z = -1.;
            self.resolution_cache.w = viewport.height;
        }

        self.projection_cache = projection_cache;
    }

    pub fn set_width_height(
        &mut self,
        projection: Projection,
        width: f32,
        height: f32,
        invert_y: bool,
    ) {
        self.set_viewport(
            projection,
            None,
            solstice::viewport::Viewport::new(0, 0, width as _, height as _),
            invert_y,
        )
    }

    pub fn set_color(&mut self, c: crate::Color) {
        self.color_cache = c.into()
    }

    pub fn bind_texture<T: solstice::texture::Texture>(&mut self, texture: T) {
        self.bind_texture_at_location(texture, 0);
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

    pub fn set_view<V: Into<mint::ColumnMatrix4<f32>>>(&mut self, view: V) {
        self.view_cache = view.into();
    }

    pub fn set_model<M: Into<mint::ColumnMatrix4<f32>>>(&mut self, model: M) {
        self.model_cache = model.into();
    }

    pub fn activate(&mut self, ctx: &mut Context) {
        use solstice::shader::RawUniformValue::{Mat4, SignedInt, Vec4};
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
        if let Some(u) = self.color_location.as_ref() {
            ctx.set_uniform_by_location(u, &Vec4(self.color_cache));
        }
        if let Some(u) = self.resolution_location.as_ref() {
            ctx.set_uniform_by_location(
                u,
                &solstice::shader::RawUniformValue::Vec4(self.resolution_cache),
            );
        }
        if let Some(projection_location) = &self.projection_location {
            ctx.set_uniform_by_location(projection_location, &Mat4(self.projection_cache));
        }
        if let Some(view_location) = &self.view_location {
            ctx.set_uniform_by_location(view_location, &Mat4(self.view_cache));
        }
        if let Some(model_location) = &self.model_location {
            ctx.set_uniform_by_location(model_location, &Mat4(self.model_cache));
        }
        if let Some(normal_location) = &self.normal_matrix_location {
            let v = nalgebra::Matrix4::from(self.view_cache) * nalgebra::Matrix4::from(self.model_cache);
            if let Some(v) = v.try_inverse() {
                let v = v.transpose();
                ctx.set_uniform_by_location(normal_location, &Mat4(v.into()))
            }
        }
    }
}

impl solstice::shader::Shader for Shader {
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
