use super::vertex::AttributeType;
use crate::{GraphicsError, ShaderKey};

#[derive(Clone, Debug)]
pub struct Attribute {
    pub name: String,
    pub size: i32,
    pub atype: AttributeType,
    pub location: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniformLocation(pub(crate) super::GLUniformLocation);

#[derive(Clone, Debug)]
pub struct Uniform {
    pub name: String,
    pub size: i32,
    pub utype: u32,
    pub location: UniformLocation,
    pub initial_data: RawUniformValue,
}

#[derive(Clone, Debug)]
pub enum RawUniformValue {
    SignedInt(i32),
    //    UnsignedInt(u32),
    Float(f32),
    Mat2(mint::ColumnMatrix2<f32>),
    Mat3(mint::ColumnMatrix3<f32>),
    Mat4(mint::ColumnMatrix4<f32>),
    Vec2(mint::Vector2<f32>),
    Vec3(mint::Vector3<f32>),
    Vec4(mint::Vector4<f32>),
    IntVec2(mint::Vector2<i32>),
    IntVec3(mint::Vector3<i32>),
    IntVec4(mint::Vector4<i32>),
    //    UnsignedIntVec2([u32; 2]),
    //    UnsignedIntVec3([u32; 3]),
    //    UnsignedIntVec4([u32; 4]),
}

macro_rules! raw_uniform_conv {
    ($from:ty, $to:ident) => {
        impl From<$from> for RawUniformValue {
            fn from(v: $from) -> Self {
                RawUniformValue::$to(v)
            }
        }

        impl std::convert::TryInto<$from> for RawUniformValue {
            type Error = &'static str;

            fn try_into(self) -> Result<$from, Self::Error> {
                match self {
                    RawUniformValue::$to(v) => Ok(v),
                    _ => Err("RawUniformValue::$to cannot be converted to $from."),
                }
            }
        }
    };
}

raw_uniform_conv!(i32, SignedInt);
raw_uniform_conv!(f32, Float);
raw_uniform_conv!(mint::ColumnMatrix2<f32>, Mat2);
raw_uniform_conv!(mint::ColumnMatrix3<f32>, Mat3);
raw_uniform_conv!(mint::ColumnMatrix4<f32>, Mat4);
raw_uniform_conv!(mint::Vector2<f32>, Vec2);
raw_uniform_conv!(mint::Vector3<f32>, Vec3);
raw_uniform_conv!(mint::Vector4<f32>, Vec4);
raw_uniform_conv!(mint::Vector2<i32>, IntVec2);
raw_uniform_conv!(mint::Vector3<i32>, IntVec3);
raw_uniform_conv!(mint::Vector4<i32>, IntVec4);

#[derive(Debug)]
pub enum ShaderError {
    VertexCompileError(String),
    FragmentCompileError(String),
    LinkError(String),
    ResourceCreationError,
}

impl std::fmt::Display for ShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ShaderError {}

#[derive(Clone)]
pub struct DynamicShader {
    inner: super::ShaderKey,
    attributes: Vec<Attribute>,
    uniforms: Vec<Uniform>,
}

impl std::cmp::PartialEq for DynamicShader {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl DynamicShader {
    pub fn new(
        gl: &mut super::Context,
        vertex_source: &str,
        fragment_source: &str,
    ) -> Result<Self, GraphicsError> {
        let inner = gl
            .new_shader(vertex_source, fragment_source)
            .map_err(GraphicsError::ShaderError)?;
        let attributes = gl.get_shader_attributes(inner);
        let uniforms = gl.get_shader_uniforms(inner);

        Ok(Self {
            inner,
            attributes,
            uniforms,
        })
    }

    pub fn handle(&self) -> super::ShaderKey {
        self.inner
    }

    pub fn get_attribute_by_name(&self, name: &str) -> Option<&Attribute> {
        self.attributes
            .iter()
            .find(|attribute| attribute.name == name)
    }

    pub fn get_uniform_by_name(&self, name: &str) -> Option<&Uniform> {
        self.uniforms.iter().find(|uniform| uniform.name == name)
    }

    pub fn create_source(vertex: &str, fragment: &str) -> (String, String) {
        let vertex = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            GLSL_VERSION, SYNTAX, VERTEX_HEADER, FUNCTIONS, LINE_PRAGMA, vertex
        );
        let fragment = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            GLSL_VERSION, SYNTAX, FRAG_HEADER, FUNCTIONS, LINE_PRAGMA, fragment
        );
        (vertex, fragment)
    }
}

impl Shader for DynamicShader {
    fn handle(&self) -> ShaderKey {
        self.inner
    }

    fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }

    fn uniforms(&self) -> &[Uniform] {
        &self.uniforms
    }
}

#[cfg(target_arch = "wasm32")]
const GLSL_VERSION: &str = "#version 100";

#[cfg(not(target_arch = "wasm32"))]
const GLSL_VERSION: &str = "#version 330 core";

#[cfg(target_arch = "wasm32")]
const LINE_PRAGMA: &str = "#line 1";

#[cfg(not(target_arch = "wasm32"))]
const LINE_PRAGMA: &str = "#line 1";

const SYNTAX: &str = r#"
#if !defined(GL_ES) && __VERSION__ < 140
    #define lowp
    #define mediump
    #define highp
#endif

#if defined(VERTEX) || __VERSION__ > 100 || defined(GL_FRAGMENT_PRECISION_HIGH)
	#define SOLSTICE_HIGHP_OR_MEDIUMP highp
#else
	#define SOLSTICE_HIGHP_OR_MEDIUMP mediump
#endif

#define extern uniform
#ifdef GL_EXT_texture_array
    #extension GL_EXT_texture_array : enable
#endif
#ifdef GL_OES_texture_3D
    #extension GL_OES_texture_3D : enable
#endif
#ifdef GL_OES_standard_derivatives
    #extension GL_OES_standard_derivatives : enable
#endif"#;

const FUNCTIONS: &str = r#"
#ifdef GL_ES
	#if __VERSION__ >= 300 || defined(GL_EXT_texture_array)
		precision lowp sampler2DArray;
	#endif
	#if __VERSION__ >= 300 || defined(GL_OES_texture_3D)
		precision lowp sampler3D;
	#endif
	#if __VERSION__ >= 300
		precision lowp sampler2DShadow;
		precision lowp samplerCubeShadow;
		precision lowp sampler2DArrayShadow;
	#endif
#endif

#if __VERSION__ >= 130
    #define texture2D Texel
    #define texture3D Texel
    #define textureCube Texel
    #define texture2DArray Texel
    #define solstice_texture2D texture
    #define solstice_texture3D texture
    #define solstice_textureCube texture
    #define solstice_texture2DArray texture
#else
    #define solstice_texture2D texture2D
    #define solstice_texture3D texture3D
    #define solstice_textureCube textureCube
    #define solstice_texture2DArray texture2DArray
#endif
vec4 Texel(sampler2D s, vec2 c) { return solstice_texture2D(s, c); }
vec4 Texel(samplerCube s, vec3 c) { return solstice_textureCube(s, c); }
#if __VERSION__ > 100 || defined(GL_OES_texture_3D)
    vec4 Texel(sampler3D s, vec3 c) { return solstice_texture3D(s, c); }
#endif
#if __VERSION__ >= 130 || defined(GL_EXT_texture_array)
    vec4 Texel(sampler2DArray s, vec3 c) { return solstice_texture2DArray(s, c); }
#endif
#ifdef PIXEL
    vec4 Texel(sampler2D s, vec2 c, float b) { return solstice_texture2D(s, c, b); }
    vec4 Texel(samplerCube s, vec3 c, float b) { return solstice_textureCube(s, c, b); }
    #if __VERSION__ > 100 || defined(GL_OES_texture_3D)
        vec4 Texel(sampler3D s, vec3 c, float b) { return solstice_texture3D(s, c, b); }
    #endif
    #if __VERSION__ >= 130 || defined(GL_EXT_texture_array)
        vec4 Texel(sampler2DArray s, vec3 c, float b) { return solstice_texture2DArray(s, c, b); }
    #endif
#endif
#define texture solstice_texture"#;

const VERTEX_HEADER: &str = r#"
#define VERTEX

#if __VERSION__ >= 130
	#define attribute in
	#define varying out
#endif"#;

const FRAG_HEADER: &str = r#"
#define FRAGMENT

#ifdef GL_ES
    precision mediump float;
#endif

#if __VERSION__ >= 130
    #define varying in
    layout(location = 0) out vec4 fragColor;
#else
    #define fragColor gl_FragColor
#endif"#;

pub trait UniformTrait {
    type Value;
    const NAME: &'static str;

    fn get_location(&self) -> Option<&UniformLocation>;
    fn update(&self, ctx: &mut super::Context, v: Self::Value)
    where
        Self::Value: std::convert::TryInto<RawUniformValue>,
    {
        use std::convert::TryInto;
        if let Some(location) = self.get_location() {
            if let Ok(value) = v.try_into() {
                ctx.set_uniform_by_location(location, &value)
            }
        }
    }
}

pub trait CachedUniformTrait: UniformTrait {
    fn get_cache(&mut self) -> &mut Self::Value;
}

pub trait Shader {
    fn handle(&self) -> super::ShaderKey;
    fn attributes(&self) -> &[Attribute];
    fn uniforms(&self) -> &[Uniform];
}

pub trait UniformGetter<U: UniformTrait> {
    fn get_uniform(&self) -> &U;
}

pub trait UniformGetterMut<U: UniformTrait>: UniformGetter<U> {
    fn get_uniform_mut(&mut self) -> &mut U;
}

pub trait BasicUniformSetter {
    fn set_uniform<U>(&mut self, gl: &mut super::Context, value: <U as UniformTrait>::Value)
    where
        Self: UniformGetter<U>,
        U: UniformTrait,
        <U as UniformTrait>::Value: Into<RawUniformValue>,
    {
        let uniform = self.get_uniform();
        if let Some(location) = uniform.get_location() {
            gl.set_uniform_by_location(location, &value.into());
        }
    }

    fn bind_texture<U, T>(
        &mut self,
        gl: &mut super::Context,
        texture: T,
        texture_unit: <U as UniformTrait>::Value,
    ) where
        Self: UniformGetter<U>,
        U: UniformTrait,
        <U as UniformTrait>::Value: Copy + Into<super::TextureUnit> + Into<RawUniformValue>,
        T: super::texture::Texture,
    {
        let uniform = self.get_uniform();
        if uniform.get_location().is_some() {
            gl.bind_texture_to_unit(
                texture.get_texture_type(),
                texture.get_texture_key(),
                texture_unit.into(),
            );
            self.set_uniform(gl, texture_unit);
        }
    }
}

pub trait CachedUniformSetter: BasicUniformSetter {
    fn set_uniform_cached<U>(&mut self, gl: &mut super::Context, value: <U as UniformTrait>::Value)
    where
        Self: UniformGetterMut<U>,
        U: CachedUniformTrait,
        <U as UniformTrait>::Value: Into<RawUniformValue> + PartialEq + Copy,
    {
        if value.ne(self.get_uniform_mut().get_cache()) {
            let uniform = self.get_uniform();
            if let Some(location) = uniform.get_location() {
                gl.set_uniform_by_location(location, &value.into());
            }
            *self.get_uniform_mut().get_cache() = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    #[test]
    fn uniform_conv() {
        let a = mint::Vector2 { x: 1., y: 2. };
        let b: RawUniformValue = a.into();
        let c: mint::Vector2<f32> = b.try_into().unwrap();
        assert_eq!(a, c);
    }
}
