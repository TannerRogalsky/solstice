use glow::HasContext;

use super::{vertex::AttributeType, GLProgram};
use std::collections::hash_map::HashMap;

fn glenum_to_attribute_type(atype: u32) -> AttributeType {
    match atype {
        glow::FLOAT => AttributeType::F32,
        glow::FLOAT_VEC2 => AttributeType::F32F32,
        glow::FLOAT_VEC3 => AttributeType::F32F32F32,
        glow::FLOAT_VEC4 => AttributeType::F32F32F32F32,
        glow::FLOAT_MAT2 => AttributeType::F32x2x2,
        glow::FLOAT_MAT3 => AttributeType::F32x3x3,
        glow::FLOAT_MAT4 => AttributeType::F32x4x4,
        v => panic!("Unknown value returned by OpenGL attribute type: {}", v),
    }
}

#[derive(Clone, Debug)]
pub struct Attribute {
    pub name: String,
    pub size: i32,
    pub atype: AttributeType,
    pub location: u32,
}

pub struct Uniform {
    pub name: String,
    pub size: i32,
    pub utype: u32,
    pub location: <glow::Context as HasContext>::UniformLocation,
}

pub enum RawUniformValue {
    SignedInt(i32),
    //    UnsignedInt(u32),
    Float(f32),
    /// 2x2 column-major matrix.
    Mat2([f32; 4]),
    /// 3x3 column-major matrix.
    Mat3([f32; 9]),
    /// 4x4 column-major matrix.
    Mat4([f32; 16]),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    IntVec2([i32; 2]),
    IntVec3([i32; 3]),
    IntVec4([i32; 4]),
    //    UnsignedIntVec2([u32; 2]),
    //    UnsignedIntVec3([u32; 3]),
    //    UnsignedIntVec4([u32; 4]),
}

pub struct Shader {
    program: GLProgram,
    attributes: Vec<Attribute>,
    uniforms: HashMap<String, Uniform>,
}

impl Shader {
    pub fn new(
        gl: &glow::Context,
        vertex_source: &str,
        fragment_source: &str,
    ) -> Result<Shader, String> {
        let mut attributes = Vec::new();
        let mut uniforms = HashMap::new();

        let program = unsafe {
            let vertex = gl.create_shader(glow::VERTEX_SHADER).unwrap();
            gl.shader_source(vertex, vertex_source);
            gl.compile_shader(vertex);
            if !gl.get_shader_compile_status(vertex) {
                let err = Err(gl.get_shader_info_log(vertex));
                gl.delete_shader(vertex);
                return err;
            }
            let fragment = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            gl.shader_source(fragment, fragment_source);
            gl.compile_shader(fragment);
            if !gl.get_shader_compile_status(fragment) {
                let err = Err(gl.get_shader_info_log(fragment));
                gl.delete_shader(fragment);
                return err;
            }
            let program = gl.create_program().unwrap();
            gl.attach_shader(program, vertex);
            gl.attach_shader(program, fragment);
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                let err = Err(gl.get_program_info_log(program));
                gl.delete_program(program);
                return err;
            }

            for index in 0..gl.get_active_attributes(program) {
                let glow::ActiveAttribute { name, size, atype } =
                    gl.get_active_attribute(program, index).unwrap();
                let location = gl.get_attrib_location(program, name.as_str()) as u32;
                attributes.push(Attribute {
                    name,
                    size,
                    atype: glenum_to_attribute_type(atype),
                    location,
                });
            }
            attributes.sort_by(|a, b| a.location.partial_cmp(&b.location).unwrap());

            for index in 0..gl.get_active_uniforms(program) {
                let glow::ActiveUniform { name, size, utype } =
                    gl.get_active_uniform(program, index).unwrap();
                let location = gl.get_uniform_location(program, name.as_str()).unwrap();
                uniforms.insert(
                    name.clone(),
                    Uniform {
                        name,
                        size,
                        utype,
                        location,
                    },
                );
            }

            program
        };

        Ok(Self {
            program,
            attributes,
            uniforms,
        })
    }

    pub fn handle(&self) -> GLProgram {
        self.program
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
