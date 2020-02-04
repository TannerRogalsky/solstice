use super::vertex::AttributeType;
use std::collections::hash_map::HashMap;

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
}

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

#[derive(Clone)]
pub struct Shader {
    inner: super::ShaderKey,
    attributes: Vec<Attribute>,
    uniforms: HashMap<String, Uniform>,
}

impl std::cmp::PartialEq for Shader {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl Shader {
    pub fn new(
        gl: &mut super::Context,
        vertex_source: &str,
        fragment_source: &str,
    ) -> Result<Shader, ShaderError> {
        let inner = gl.new_shader(vertex_source, fragment_source)?;
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

    pub fn attributes(&self) -> &Vec<Attribute> {
        &self.attributes
    }

    pub fn get_attribute_by_name(&self, name: &str) -> Option<&Attribute> {
        self.attributes
            .iter()
            .find(|&attribute| attribute.name.as_str() == name)
    }

    pub fn get_uniform_by_name(&self, name: &str) -> Option<&Uniform> {
        self.uniforms.get(name)
    }

    pub fn create_source(vertex: &str, fragment: &str) -> (String, String) {
        let vertex = format!(
            "{}\n{}\n{}\n{}\n{}",
            GLSL_VERSION, SYNTAX, VERTEX_HEADER, LINE_PRAGMA, vertex
        );
        let fragment = format!(
            "{}\n{}\n{}\n{}\n{}",
            GLSL_VERSION, SYNTAX, FRAG_HEADER, LINE_PRAGMA, fragment
        );
        (vertex, fragment)
    }
}

#[cfg(target_arch = "wasm32")]
const GLSL_VERSION: &str = "#version 100";

#[cfg(not(target_arch = "wasm32"))]
const GLSL_VERSION: &str = "#version 330 core";

#[cfg(target_arch = "wasm32")]
const LINE_PRAGMA: &str = "#line 1";

#[cfg(not(target_arch = "wasm32"))]
const LINE_PRAGMA: &str = "#line 0";

const SYNTAX: &str = r#"
#if !defined(GL_ES) && __VERSION__ < 140
    #define lowp
    #define mediump
    #define highp
#endif

#if __VERSION__ >= 130
    #define Texel texture
#else
    #define Texel texture2D
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

    fn get_location(&self) -> Option<&UniformLocation>;
    fn get_name() -> &'static str {
        ""
    }
}

pub trait ShaderTrait {
    fn get_inner(&self) -> &super::shader::Shader;

    fn bind(&self, ctx: &mut super::Context) {
        ctx.use_shader(Some(self.get_inner()))
    }

    fn location(shader: &super::shader::Shader, name: &str) -> Option<UniformLocation> {
        shader
            .get_uniform_by_name(name)
            .map(|uniform| uniform.location.clone())
    }
}

pub trait UniformGetterMut<U>
where
    U: UniformTrait,
{
    fn get_uniform_mut(&mut self) -> &mut U;
}

pub trait BasicUniformSetter {
    fn set_uniform<U>(&mut self, gl: &mut super::Context, value: <U as UniformTrait>::Value)
    where
        Self: UniformGetterMut<U>,
        U: UniformTrait,
        <U as UniformTrait>::Value: Into<RawUniformValue>,
    {
        let uniform = self.get_uniform_mut();
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
        Self: UniformGetterMut<U>,
        U: UniformTrait,
        <U as UniformTrait>::Value: Copy + Into<super::TextureUnit> + Into<RawUniformValue>,
        T: super::texture::Texture,
    {
        let uniform = self.get_uniform_mut();
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
