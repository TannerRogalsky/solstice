pub use glow;

#[cfg(feature = "derive")]
extern crate solstice_derive;

pub mod buffer;
pub mod canvas;
pub mod image;
pub mod mesh;
pub mod quad_batch;
pub mod shader;
pub mod texture;
pub mod vertex;
pub mod viewport;

mod gl;

use glow::HasContext;
use slotmap::SlotMap;
use std::{
    fmt::{Debug, Error, Formatter},
    str::FromStr,
};

#[derive(Debug)]
pub enum GraphicsError {
    ShaderError(shader::ShaderError),
    TextureError,
    BufferError,
    FramebufferError,
    RenderbufferError,
}

impl std::fmt::Display for GraphicsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for GraphicsError {}

type GLContext = glow::Context;

type GLBuffer = <GLContext as HasContext>::Buffer;
type GLProgram = <GLContext as HasContext>::Program;
type GLTexture = <GLContext as HasContext>::Texture;
type GLFramebuffer = <GLContext as HasContext>::Framebuffer;
type GLRenderbuffer = <GLContext as HasContext>::Renderbuffer;
type GLUniformLocation = <GLContext as HasContext>::UniformLocation;

slotmap::new_key_type! {
    pub struct ShaderKey;
    pub struct BufferKey;
    pub struct TextureKey;
    pub struct FramebufferKey;
    pub struct RenderbufferKey;
}

pub struct DebugGroup<'a> {
    ctx: &'a GLContext,
}

impl<'a> DebugGroup<'a> {
    pub fn new(ctx: &'a GLContext, message: &str) -> Self {
        if ctx.supports_debug() {
            unsafe {
                ctx.push_debug_group(glow::DEBUG_SOURCE_APPLICATION, 0, message);
            }
        }
        Self { ctx }
    }
}

impl<'a> Drop for DebugGroup<'a> {
    fn drop(&mut self) {
        if self.ctx.supports_debug() {
            unsafe {
                self.ctx.pop_debug_group();
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PixelFormat {
    Unknown,

    // "regular" formats
    LUMINANCE,
    RG8,
    RGB8,
    RGBA8,
    SRGBA8,
    R16,
    RG16,
    RGBA16,
    R16F,
    RG16F,
    RGBA16F,
    R32F,
    RG32F,
    RGBA32F,
    Alpha,

    // depth/stencil formats
    Stencil8,
    Depth16,
    Depth24,
    Depth32F,
    Depth24Stencil8,
    Depth32fStencil8,
}

fn target_to_index(target: canvas::Target) -> usize {
    match target {
        canvas::Target::Draw => 0,
        canvas::Target::Read => 1,
        canvas::Target::All => 0,
    }
}

fn buffer_type_to_index(buffer_type: buffer::BufferType) -> usize {
    match buffer_type {
        buffer::BufferType::Vertex => 0,
        buffer::BufferType::Index => 1,
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VertexWinding {
    ClockWise,
    CounterClockWise,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DepthFunction {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

impl DepthFunction {
    pub fn to_gl(&self) -> u32 {
        match self {
            DepthFunction::Never => glow::NEVER,
            DepthFunction::Less => glow::LESS,
            DepthFunction::Equal => glow::EQUAL,
            DepthFunction::LessEqual => glow::LEQUAL,
            DepthFunction::Greater => glow::GREATER,
            DepthFunction::NotEqual => glow::NOTEQUAL,
            DepthFunction::GreaterEqual => glow::GEQUAL,
            DepthFunction::Always => glow::ALWAYS,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CullFace {
    Back,
    Front,
    FrontAndBack,
}

impl CullFace {
    pub fn to_gl(&self) -> u32 {
        match self {
            CullFace::Back => glow::BACK,
            CullFace::Front => glow::FRONT,
            CullFace::FrontAndBack => glow::FRONT_AND_BACK,
        }
    }
}

pub enum Feature {
    DepthTest(DepthFunction),
    CullFace(CullFace, VertexWinding),
}

struct GLConstants {
    max_vertex_attributes: usize,
    max_texture_units: usize,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DrawMode {
    Points,
    Lines,
    LineLoop,
    LineStrip,
    Triangles,
    TriangleStrip,
    TriangleFan,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct TextureUnit {
    index: u32,
    gl: u32,
}

impl From<u32> for TextureUnit {
    fn from(v: u32) -> Self {
        let inner = match v {
            0 => glow::TEXTURE0,
            1 => glow::TEXTURE1,
            2 => glow::TEXTURE2,
            3 => glow::TEXTURE3,
            4 => glow::TEXTURE4,
            5 => glow::TEXTURE5,
            6 => glow::TEXTURE6,
            7 => glow::TEXTURE7,
            8 => glow::TEXTURE8,
            9 => glow::TEXTURE9,
            10 => glow::TEXTURE10,
            11 => glow::TEXTURE11,
            12 => glow::TEXTURE12,
            13 => glow::TEXTURE13,
            14 => glow::TEXTURE14,
            15 => glow::TEXTURE15,
            16 => glow::TEXTURE16,
            17 => glow::TEXTURE17,
            18 => glow::TEXTURE18,
            19 => glow::TEXTURE19,
            20 => glow::TEXTURE20,
            21 => glow::TEXTURE21,
            22 => glow::TEXTURE22,
            23 => glow::TEXTURE23,
            24 => glow::TEXTURE24,
            25 => glow::TEXTURE25,
            26 => glow::TEXTURE26,
            27 => glow::TEXTURE27,
            28 => glow::TEXTURE28,
            29 => glow::TEXTURE29,
            30 => glow::TEXTURE30,
            31 => glow::TEXTURE31,
            _ => panic!("unsupported texture unit: {}", v),
        };
        TextureUnit {
            index: v,
            gl: inner,
        }
    }
}

impl From<i32> for TextureUnit {
    fn from(v: i32) -> Self {
        (v as u32).into()
    }
}

impl From<usize> for TextureUnit {
    fn from(v: usize) -> Self {
        (v as u32).into()
    }
}

#[derive(Copy, Clone, Default)]
pub struct GLVersion {
    major: u32,
    minor: u32,
    gles: bool,
}

impl FromStr for GLVersion {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // the webgl property is about whether the string was parsed as webgl
        // rather than whether the gl implementation is webgl
        let (major, minor, gles, webgl) = if s.starts_with("WebGL ") {
            (s.chars().nth(6), s.chars().nth(8), true, true)
        } else if s.contains("OpenGL ES ") {
            (s.chars().nth(10), s.chars().nth(12), true, false)
        } else {
            (s.chars().next(), s.chars().nth(2), false, false)
        };

        // this conflates WebGL X with OpenGL ES X+1 but
        // it's done intentionally so it's okay?
        let major_incr = if webgl { 1 } else { 0 };

        let major = major.and_then(|c| c.to_digit(10));
        let minor = minor.and_then(|c| c.to_digit(10));
        match (major, minor) {
            (Some(major), Some(minor)) => Ok(Self {
                major: major + major_incr as u32,
                minor: minor as u32,
                gles,
            }),
            _ => Err(()),
        }
    }
}

impl Debug for GLVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "GLVersion {{ major: {}, minor: {}, ES: {} }}",
            self.major, self.minor, self.gles
        )
    }
}

// a caching, convenience and safety layer around glow
pub struct Context {
    ctx: GLContext,
    version: GLVersion,
    gl_constants: GLConstants,
    shaders: SlotMap<ShaderKey, GLProgram>,
    active_shader: Option<ShaderKey>,
    buffers: SlotMap<BufferKey, GLBuffer>,
    active_buffers: [Option<BufferKey>; 2],
    textures: SlotMap<TextureKey, GLTexture>,
    bound_textures: Vec<Vec<Option<GLTexture>>>,
    framebuffers: SlotMap<FramebufferKey, GLFramebuffer>,
    active_framebuffer: [Option<FramebufferKey>; 2],
    renderbuffers: SlotMap<RenderbufferKey, GLRenderbuffer>,
    active_renderbuffer: Option<RenderbufferKey>,
    current_texture_unit: TextureUnit,
    current_viewport: viewport::Viewport<i32>,
    current_scissor: Option<viewport::Viewport<i32>>,
    enabled_attributes: u32, // a bitmask that represents the vertex attribute state
}

impl Context {
    pub fn new(ctx: GLContext) -> Self {
        let gl_constants = GLConstants {
            max_vertex_attributes: unsafe {
                ctx.get_parameter_i32(glow::MAX_VERTEX_ATTRIBS) as usize
            },
            max_texture_units: unsafe {
                ctx.get_parameter_i32(glow::MAX_COMBINED_TEXTURE_IMAGE_UNITS) as usize
            },
        };

        let bound_textures = texture::TextureType::enumerate()
            .iter()
            .map(|_tt| vec![None; gl_constants.max_texture_units])
            .collect();

        for texture_unit in 0..gl_constants.max_texture_units {
            unsafe {
                ctx.active_texture(glow::TEXTURE0 + texture_unit as u32);
                // do this for every supported texture type
                for texture_type in texture::TextureType::enumerate() {
                    if texture_type.is_supported() {
                        ctx.bind_texture(gl::texture::to_gl(*texture_type), None);
                    }
                }
            }
        }
        unsafe { ctx.active_texture(glow::TEXTURE0) }
        unsafe {
            // TODO: this should be left to the consumer
            ctx.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
            ctx.enable(glow::BLEND);
            ctx.blend_equation(glow::FUNC_ADD);
            ctx.blend_func_separate(
                glow::SRC_ALPHA,
                glow::ONE_MINUS_SRC_ALPHA,
                glow::ONE,
                glow::ONE_MINUS_SRC_ALPHA,
            );

            ctx.bind_vertex_array(ctx.create_vertex_array().ok());
        }

        let version = {
            let str_version = unsafe { ctx.get_parameter_string(glow::VERSION) };
            str_version.parse::<GLVersion>().unwrap_or_default()
        };

        let mut ctx = Self {
            ctx,
            version,
            gl_constants,
            shaders: SlotMap::with_key(),
            active_shader: None,
            buffers: SlotMap::with_key(),
            active_buffers: [None; 2],
            textures: SlotMap::with_key(),
            bound_textures,
            framebuffers: SlotMap::with_key(),
            active_framebuffer: [None; 2],
            renderbuffers: SlotMap::with_key(),
            active_renderbuffer: None,
            current_texture_unit: 0.into(),
            current_viewport: viewport::Viewport::default(),
            current_scissor: None,
            enabled_attributes: std::u32::MAX,
        };
        ctx.set_vertex_attributes(0, &[]);
        ctx
    }

    pub fn enable(&mut self, feature: Feature) {
        match feature {
            Feature::DepthTest(func) => unsafe {
                self.ctx.enable(glow::DEPTH_TEST);
                self.ctx.depth_func(func.to_gl());
            },
            Feature::CullFace(cull_face, winding_order) => unsafe {
                self.ctx.enable(glow::CULL_FACE);
                self.ctx.cull_face(cull_face.to_gl());
                self.ctx
                    .front_face(gl::vertex_winding::to_gl(winding_order));
            },
        }
    }

    pub fn disable(&mut self, feature: Feature) {
        match feature {
            Feature::DepthTest(_) => unsafe { self.ctx.disable(glow::DEPTH_TEST) },
            Feature::CullFace(_, _) => unsafe { self.ctx.disable(glow::CULL_FACE) },
        }
    }

    pub fn new_debug_group(&self, message: &str) -> DebugGroup {
        DebugGroup::new(&self.ctx, message)
    }

    pub fn new_buffer(
        &mut self,
        size: usize,
        buffer_type: buffer::BufferType,
        usage: buffer::Usage,
        initial_data: Option<&[u8]>,
    ) -> Result<BufferKey, GraphicsError> {
        let vbo = unsafe {
            let vbo = self
                .ctx
                .create_buffer()
                .map_err(|_| GraphicsError::BufferError)?;
            self.ctx.bind_buffer(buffer_type.into(), Some(vbo));
            if let Some(initial_data) = initial_data {
                self.ctx
                    .buffer_data_u8_slice(buffer_type.into(), initial_data, usage.to_gl());
            } else {
                self.ctx
                    .buffer_data_size(buffer_type.into(), size as _, usage.to_gl());
            }

            vbo
        };
        let buffer_key = self.buffers.insert(vbo);
        self.active_buffers[buffer_type_to_index(buffer_type)] = Some(buffer_key);
        Ok(buffer_key)
    }

    pub fn destroy_buffer(&mut self, buffer: &buffer::Buffer) {
        if let Some(gl_buffer) = self.buffers.remove(buffer.handle()) {
            unsafe {
                self.ctx.delete_buffer(gl_buffer);
            }
        }
    }

    pub fn bind_buffer(&mut self, buffer_key: BufferKey, buffer_type: buffer::BufferType) {
        if let Some(&vbo) = self.buffers.get(buffer_key) {
            let buffer_index = buffer_type_to_index(buffer_type);
            match self.active_buffers.get_mut(buffer_index) {
                Some(Some(active_buffer)) => {
                    if active_buffer != &buffer_key {
                        *active_buffer = buffer_key;
                        unsafe { self.ctx.bind_buffer(buffer_type.into(), Some(vbo)) };
                    }
                }
                _ => {
                    self.active_buffers[buffer_index] = Some(buffer_key);
                    unsafe { self.ctx.bind_buffer(buffer_type.into(), Some(vbo)) };
                }
            }
        }
    }

    pub fn buffer_static_draw(&self, buffer: &buffer::Buffer, data: &[u8], offset: usize) {
        let target = buffer.buffer_type().into();
        unsafe {
            self.ctx
                .buffer_sub_data_u8_slice(target, offset as i32, data)
        }
    }

    fn buffer_stream_draw(&self, map: &buffer::MappedBuffer) {
        let buffer = map.inner();
        let target = buffer.buffer_type().into();
        let data = map.memory_map();

        unsafe {
            // "orphan" current buffer to avoid implicit synchronisation on the GPU:
            // http://www.seas.upenn.edu/~pcozzi/OpenGLInsights/OpenGLInsights-AsynchronousBufferTransfers.pdf
            self.ctx
                .buffer_data_size(target, buffer.size() as i32, buffer.usage().to_gl());
            self.ctx.buffer_sub_data_u8_slice(target, 0, data);
        }
    }

    pub fn unmap_buffer(&mut self, map: &mut buffer::MappedBuffer) {
        let buffer = map.inner();
        self.bind_buffer(buffer.handle(), buffer.buffer_type());
        if self.buffers.get(buffer.handle()).is_some() {
            if let Some(modified_range) = map.modified_range() {
                let modified_offset =
                    std::cmp::min(modified_range.offset, buffer.size().saturating_sub(1));
                let modified_size = std::cmp::min(
                    modified_range.size,
                    buffer.size().saturating_sub(modified_range.offset),
                );
                match buffer.usage() {
                    buffer::Usage::Stream => self.buffer_stream_draw(map),
                    buffer::Usage::Static => self.buffer_static_draw(
                        buffer,
                        &map.memory_map()[modified_offset..(modified_size + modified_offset)],
                        modified_offset,
                    ),
                    buffer::Usage::Dynamic => {
                        if modified_size >= buffer.size() / 3 {
                            self.buffer_stream_draw(map);
                        } else {
                            self.buffer_static_draw(
                                buffer,
                                &map.memory_map()
                                    [modified_offset..(modified_size + modified_offset)],
                                modified_offset,
                            );
                        }
                    }
                }
            }
        } else {
            log::warn!(
                "attempted to unmap non-existant buffer, {:?}",
                buffer.handle()
            );
        }
    }

    pub fn new_shader(
        &mut self,
        vertex_source: &str,
        fragment_source: &str,
    ) -> Result<ShaderKey, shader::ShaderError> {
        use shader::*;
        let program = unsafe {
            let gl = &self.ctx;
            let vertex = gl
                .create_shader(glow::VERTEX_SHADER)
                .map_err(|_| ShaderError::ResourceCreationError)?;
            gl.shader_source(vertex, vertex_source);
            gl.compile_shader(vertex);
            if !gl.get_shader_compile_status(vertex) {
                let err = Err(ShaderError::VertexCompileError(
                    gl.get_shader_info_log(vertex),
                ));
                gl.delete_shader(vertex);
                return err;
            }
            let fragment = gl
                .create_shader(glow::FRAGMENT_SHADER)
                .expect("Failed to create Fragment shader.");
            gl.shader_source(fragment, fragment_source);
            gl.compile_shader(fragment);
            if !gl.get_shader_compile_status(fragment) {
                let err = Err(ShaderError::FragmentCompileError(
                    gl.get_shader_info_log(fragment),
                ));
                gl.delete_shader(fragment);
                return err;
            }
            let program = gl.create_program().expect("Failed to create program.");
            gl.attach_shader(program, vertex);
            gl.attach_shader(program, fragment);
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                let err = Err(ShaderError::LinkError(gl.get_program_info_log(program)));
                gl.delete_program(program);
                return err;
            }

            program
        };

        Ok(self.shaders.insert(program))
    }

    pub fn get_shader_attributes(&self, shader: ShaderKey) -> Vec<shader::Attribute> {
        if let Some(program) = self.shaders.get(shader).cloned() {
            let count = unsafe { self.ctx.get_active_attributes(program) };
            let mut attributes = Vec::with_capacity(count as usize);
            for index in 0..count {
                unsafe {
                    let glow::ActiveAttribute { name, size, atype } =
                        self.ctx.get_active_attribute(program, index).unwrap();
                    if let Some(location) = self.ctx.get_attrib_location(program, name.as_str()) {
                        // specifically this is for gl_InstanceID
                        attributes.push(shader::Attribute {
                            name,
                            size,
                            atype: gl::attribute::from_gl(atype),
                            location,
                        });
                    }
                }
            }
            attributes.sort_by(|a, b| a.location.partial_cmp(&b.location).unwrap());
            attributes
        } else {
            Vec::new()
        }
    }

    pub fn get_shader_uniforms(&self, shader: ShaderKey) -> Vec<shader::Uniform> {
        unsafe fn get_initial_uniform_data(
            gl: &glow::Context,
            utype: u32,
            program: GLProgram,
            location: &GLUniformLocation,
        ) -> shader::RawUniformValue {
            use shader::RawUniformValue;
            macro_rules! get_uniform_data {
                (f32, 1, $uni_ty:ident, $gl:expr, $program:expr, $location:expr) => {{
                    let mut data = [0.; 1];
                    $gl.get_uniform_f32($program, $location, &mut data);
                    RawUniformValue::$uni_ty(data[0].into())
                }};
                (i32, 1, $uni_ty:ident, $gl:expr, $program:expr, $location:expr) => {{
                    let mut data = [0; 1];
                    $gl.get_uniform_i32($program, $location, &mut data);
                    RawUniformValue::$uni_ty(data[0].into())
                }};
                (f32, $data_size:expr, $uni_ty:ident, $gl:expr, $program:expr, $location:expr) => {{
                    let mut data = [0.; $data_size];
                    $gl.get_uniform_f32($program, $location, &mut data);
                    RawUniformValue::$uni_ty(data.into())
                }};
                (i32, $data_size:expr, $uni_ty:ident, $gl:expr, $program:expr, $location:expr) => {{
                    let mut data = [0; $data_size];
                    $gl.get_uniform_i32($program, $location, &mut data);
                    RawUniformValue::$uni_ty(data.into())
                }};
            }

            match utype {
                glow::FLOAT => get_uniform_data!(f32, 1, Float, gl, program, location),
                glow::FLOAT_VEC2 => get_uniform_data!(f32, 2, Vec2, gl, program, location),
                glow::FLOAT_VEC3 => get_uniform_data!(f32, 3, Vec3, gl, program, location),
                glow::FLOAT_VEC4 => get_uniform_data!(f32, 4, Vec4, gl, program, location),
                glow::FLOAT_MAT2 => get_uniform_data!(f32, 4, Mat2, gl, program, location),
                glow::FLOAT_MAT3 => get_uniform_data!(f32, 9, Mat3, gl, program, location),
                glow::FLOAT_MAT4 => get_uniform_data!(f32, 16, Mat4, gl, program, location),
                glow::INT | glow::SAMPLER_2D | glow::SAMPLER_CUBE => {
                    get_uniform_data!(i32, 1, SignedInt, gl, program, location)
                }
                glow::INT_VEC2 => get_uniform_data!(i32, 2, IntVec2, gl, program, location),
                glow::INT_VEC3 => get_uniform_data!(i32, 3, IntVec3, gl, program, location),
                glow::INT_VEC4 => get_uniform_data!(i32, 4, IntVec4, gl, program, location),
                _ => {
                    panic!("failed to match uniform type");
                }
            }
        }

        use shader::{Uniform, UniformLocation};
        let gl = &self.ctx;
        if let Some(program) = self.shaders.get(shader).cloned() {
            let count = unsafe { gl.get_active_uniforms(program) };
            let mut uniforms = Vec::with_capacity(count as usize);
            for index in 0..count {
                unsafe {
                    let glow::ActiveUniform { name, size, utype } =
                        gl.get_active_uniform(program, index).unwrap();
                    if size > 1 {
                        let name = name.trim_end_matches("[0]");
                        uniforms.extend((0..size).map(|i| {
                            let name = format!("{}[{}]", name, i);
                            let location = gl.get_uniform_location(program, name.as_str()).unwrap();
                            let initial_data =
                                get_initial_uniform_data(&gl, utype, program, &location);
                            let location = UniformLocation(location);
                            Uniform {
                                name,
                                size: 1,
                                utype,
                                location,
                                initial_data,
                            }
                        }));
                    } else {
                        let location = gl
                            .get_uniform_location(program, name.as_str())
                            .expect("Failed to get uniform?!");
                        let initial_data = get_initial_uniform_data(&gl, utype, program, &location);
                        let location = UniformLocation(location);
                        uniforms.push(Uniform {
                            name,
                            size,
                            utype,
                            location,
                            initial_data,
                        });
                    }
                }
            }
            uniforms
        } else {
            Default::default()
        }
    }

    pub fn destroy_shader(&mut self, shader: ShaderKey) {
        match self.shaders.remove(shader) {
            None => (),
            Some(shader) => unsafe {
                self.ctx.delete_program(shader);
            },
        }
    }

    pub fn use_shader<S: shader::Shader + ?Sized>(&mut self, shader: Option<&S>) {
        match shader {
            None => {
                if self.active_shader.is_some() {
                    self.active_shader = None;
                    unsafe {
                        self.ctx.use_program(None);
                    }
                }
            }
            Some(shader) => {
                if self.active_shader != Some(shader.handle()) {
                    match self.shaders.get(shader.handle()) {
                        None => log::warn!(
                            "Attempting to bind shader not in cache: {:?}",
                            shader.handle()
                        ),
                        Some(gl_shader) => {
                            self.active_shader = Some(shader.handle());
                            unsafe { self.ctx.use_program(Some(*gl_shader)) }
                        }
                    }
                }
            }
        }
    }

    pub fn new_texture(
        &mut self,
        texture_type: texture::TextureType,
    ) -> Result<TextureKey, GraphicsError> {
        unsafe {
            let handle = self
                .ctx
                .create_texture()
                .map_err(|_| GraphicsError::TextureError)?;
            let texture = self.textures.insert(handle);
            self.ctx.active_texture(glow::TEXTURE0);
            self.bind_texture_to_unit(texture_type, texture, 0.into());
            Ok(texture)
        }
    }

    pub fn destroy_texture(&mut self, texture_key: TextureKey) {
        match self.textures.remove(texture_key) {
            None => (),
            Some(texture) => unsafe { self.ctx.delete_texture(texture) },
        }
    }

    pub fn bind_texture_to_unit(
        &mut self,
        texture_type: texture::TextureType,
        texture_key: TextureKey,
        texture_unit: TextureUnit,
    ) {
        let TextureUnit { index, gl: unit } = texture_unit;
        let texture_unit_index = index as usize;
        match (
            self.textures.get(texture_key),
            self.bound_textures[texture_type.to_index()][texture_unit_index],
        ) {
            (Some(&texture), None) => {
                if texture_unit != self.current_texture_unit {
                    unsafe {
                        self.ctx.active_texture(unit);
                    }
                    self.current_texture_unit = texture_unit;
                }
                self.bound_textures[texture_type.to_index()][texture_unit_index] = Some(texture);
                unsafe {
                    self.ctx
                        .bind_texture(gl::texture::to_gl(texture_type), Some(texture))
                }
            }
            (Some(&texture), Some(bound_texture)) => {
                if texture != bound_texture {
                    if texture_unit != self.current_texture_unit {
                        unsafe {
                            self.ctx.active_texture(unit);
                        }
                        self.current_texture_unit = texture_unit;
                    }
                    self.bound_textures[texture_type.to_index()][texture_unit_index] =
                        Some(texture);
                    unsafe {
                        self.ctx
                            .bind_texture(gl::texture::to_gl(texture_type), Some(texture))
                    }
                }
            }
            (None, Some(_)) => {
                if texture_unit != self.current_texture_unit {
                    unsafe {
                        self.ctx.active_texture(unit);
                    }
                    self.current_texture_unit = texture_unit;
                }
                self.bound_textures[texture_type.to_index()][texture_unit_index] = None;
                unsafe {
                    self.ctx
                        .bind_texture(gl::texture::to_gl(texture_type), None)
                }
            }
            (None, None) => (),
        }
    }

    pub fn new_framebuffer(&mut self) -> Result<FramebufferKey, GraphicsError> {
        let framebuffer = unsafe {
            self.ctx
                .create_framebuffer()
                .map_err(|_| GraphicsError::FramebufferError)?
        };
        Ok(self.framebuffers.insert(framebuffer))
    }

    pub fn destroy_framebuffer(&mut self, framebuffer_key: FramebufferKey) {
        match self.framebuffers.remove(framebuffer_key) {
            None => (),
            Some(framebuffer) => unsafe { self.ctx.delete_framebuffer(framebuffer) },
        }
    }

    pub fn bind_framebuffer(
        &mut self,
        target: canvas::Target,
        framebuffer_key: Option<FramebufferKey>,
    ) {
        let target_index = target_to_index(target);
        match (framebuffer_key, self.active_framebuffer[target_index]) {
            (None, None) => (),
            (Some(framebuffer_key), None) => match self.framebuffers.get(framebuffer_key) {
                None => (),
                Some(framebuffer) => {
                    self.active_framebuffer[target_index] = Some(framebuffer_key);
                    unsafe {
                        self.ctx
                            .bind_framebuffer(target.to_gl(), Some(*framebuffer))
                    }
                }
            },
            (Some(framebuffer_key), Some(current_framebuffer_key)) => {
                if framebuffer_key != current_framebuffer_key {
                    match self.framebuffers.get(framebuffer_key) {
                        None => (),
                        Some(framebuffer) => {
                            self.active_framebuffer[target_index] = Some(framebuffer_key);
                            unsafe {
                                self.ctx
                                    .bind_framebuffer(target.to_gl(), Some(*framebuffer))
                            }
                        }
                    }
                }
            }
            (None, Some(_current_framebuffer_key)) => {
                self.active_framebuffer[target_index] = None;
                unsafe { self.ctx.bind_framebuffer(target.to_gl(), None) }
            }
        }
    }

    pub fn check_framebuffer_status(&self, target: canvas::Target) -> canvas::Status {
        match unsafe { self.ctx.check_framebuffer_status(target.to_gl()) } {
            glow::FRAMEBUFFER_COMPLETE => canvas::Status::Complete,
            glow::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => canvas::Status::IncompleteAttachment,
            glow::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => canvas::Status::MissingAttachment,
            glow::FRAMEBUFFER_UNSUPPORTED => canvas::Status::Unsupported,
            glow::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => canvas::Status::IncompleteMultisample,
            _ => canvas::Status::Unknown,
        }
    }

    pub fn get_active_framebuffer(&self, target: canvas::Target) -> Option<FramebufferKey> {
        self.active_framebuffer[target_to_index(target)]
    }

    pub fn framebuffer_texture(
        &mut self,
        target: canvas::Target,
        attachment: canvas::Attachment,
        texture_type: texture::TextureType,
        texture_key: TextureKey,
        level: u32,
    ) {
        unsafe {
            self.ctx.framebuffer_texture_2d(
                target.to_gl(),
                attachment.to_gl(),
                gl::texture::to_gl(texture_type),
                self.textures.get(texture_key).copied(),
                level as i32,
            )
        }
    }

    pub fn new_renderbuffer(&mut self) -> Result<RenderbufferKey, GraphicsError> {
        let renderbuffer = unsafe {
            self.ctx
                .create_renderbuffer()
                .map_err(|_| GraphicsError::RenderbufferError)?
        };
        Ok(self.renderbuffers.insert(renderbuffer))
    }

    pub fn bind_renderbuffer(&mut self, renderbuffer: Option<RenderbufferKey>) {
        if self.active_renderbuffer != renderbuffer {
            self.active_renderbuffer = renderbuffer;
            let gl_renderbuffer =
                renderbuffer.and_then(|renderbuffer| self.renderbuffers.get(renderbuffer).cloned());
            unsafe {
                self.ctx
                    .bind_renderbuffer(glow::RENDERBUFFER, gl_renderbuffer);
            }
        }
    }

    pub fn renderbuffer_storage(&mut self, format: PixelFormat, width: i32, height: i32) {
        let gl_format = gl::pixel_format::to_gl(format, &self.version);
        unsafe {
            self.ctx
                .renderbuffer_storage(glow::RENDERBUFFER, gl_format.internal, width, height)
        }
    }

    pub fn framebuffer_renderbuffer(
        &mut self,
        attachment: canvas::Attachment,
        renderbuffer: Option<RenderbufferKey>,
    ) {
        let gl_renderbuffer =
            renderbuffer.and_then(|renderbuffer| self.renderbuffers.get(renderbuffer).cloned());
        unsafe {
            self.ctx.framebuffer_renderbuffer(
                glow::FRAMEBUFFER,
                attachment.to_gl(),
                glow::RENDERBUFFER,
                gl_renderbuffer,
            )
        }
    }

    pub fn destroy_renderbuffer(&mut self, renderbuffer_key: RenderbufferKey) {
        match self.renderbuffers.remove(renderbuffer_key) {
            None => (),
            Some(renderbuffer) => unsafe { self.ctx.delete_renderbuffer(renderbuffer) },
        }
    }

    pub fn set_vertex_attributes(
        &mut self,
        desired: u32,
        binding_info: &[Option<mesh::BindingInfo>],
    ) {
        let diff = desired ^ self.enabled_attributes;
        for i in 0..self.gl_constants.max_vertex_attributes as u32 {
            let bit = 1 << i;

            if diff & bit != 0 {
                if desired & bit != 0 {
                    unsafe {
                        self.ctx.enable_vertex_attrib_array(i);
                    }
                } else {
                    unsafe {
                        self.ctx.disable_vertex_attrib_array(i);
                    }
                }
            }

            if desired & bit != 0 {
                let (vertex_format, stride, step, buffer_key, buffer_type) =
                    binding_info[i as usize].unwrap();
                self.bind_buffer(buffer_key, buffer_type);
                let (data_type, elements_count, _instances_count) = vertex_format.atype.to_gl();
                unsafe {
                    self.ctx.vertex_attrib_divisor(i, step);
                    use vertex::AttributeType;
                    match vertex_format.atype {
                        AttributeType::F32
                        | AttributeType::F32F32
                        | AttributeType::F32F32F32
                        | AttributeType::F32F32F32F32
                        | AttributeType::F32x2x2
                        | AttributeType::F32x3x3
                        | AttributeType::F32x4x4 => self.ctx.vertex_attrib_pointer_f32(
                            i,
                            elements_count,
                            data_type,
                            vertex_format.normalize,
                            stride as i32,
                            vertex_format.offset as i32,
                        ),
                        AttributeType::I32
                        | AttributeType::I32I32
                        | AttributeType::I32I32I32
                        | AttributeType::I32I32I32I32 => self.ctx.vertex_attrib_pointer_i32(
                            i,
                            elements_count,
                            data_type,
                            stride as i32,
                            vertex_format.offset as i32,
                        ),
                    }
                }
            }
        }

        self.enabled_attributes = desired;
    }

    pub fn set_uniform_by_location(
        &self,
        location: &shader::UniformLocation,
        data: &shader::RawUniformValue,
    ) {
        assert!(
            self.active_shader.is_some(),
            "Setting a uniform without an active shader."
        );
        use shader::RawUniformValue;
        let location = Some(&location.0);
        unsafe {
            match data {
                RawUniformValue::SignedInt(data) => self.ctx.uniform_1_i32(location, *data),
                RawUniformValue::Float(data) => self.ctx.uniform_1_f32(location, *data),
                RawUniformValue::Mat2(data) => self.ctx.uniform_matrix_2_f32_slice(
                    location,
                    false,
                    &AsRef::<[f32; 4]>::as_ref(data)[..],
                ),
                RawUniformValue::Mat3(data) => self.ctx.uniform_matrix_3_f32_slice(
                    location,
                    false,
                    &AsRef::<[f32; 9]>::as_ref(data)[..],
                ),
                RawUniformValue::Mat4(data) => self.ctx.uniform_matrix_4_f32_slice(
                    location,
                    false,
                    &AsRef::<[f32; 16]>::as_ref(data)[..],
                ),
                RawUniformValue::Vec2(data) => {
                    self.ctx.uniform_2_f32_slice(location, data.as_ref())
                }
                RawUniformValue::Vec3(data) => {
                    self.ctx.uniform_3_f32_slice(location, data.as_ref())
                }
                RawUniformValue::Vec4(data) => {
                    self.ctx.uniform_4_f32_slice(location, data.as_ref())
                }
                RawUniformValue::IntVec2(data) => {
                    self.ctx.uniform_2_i32_slice(location, data.as_ref())
                }
                RawUniformValue::IntVec3(data) => {
                    self.ctx.uniform_3_i32_slice(location, data.as_ref())
                }
                RawUniformValue::IntVec4(data) => {
                    self.ctx.uniform_4_i32_slice(location, data.as_ref())
                }
            }
        }
    }

    pub fn draw_arrays(&self, mode: DrawMode, first: i32, count: i32) {
        unsafe {
            self.ctx
                .draw_arrays(gl::draw_mode::to_gl(mode), first, count);
        }
    }

    pub fn draw_elements(&self, mode: DrawMode, count: i32, element_type: u32, offset: i32) {
        unsafe {
            self.ctx
                .draw_elements(gl::draw_mode::to_gl(mode), count, element_type, offset);
        }
    }

    pub fn draw_arrays_instanced(
        &self,
        mode: DrawMode,
        first: i32,
        count: i32,
        instance_count: i32,
    ) {
        unsafe {
            self.ctx
                .draw_arrays_instanced(gl::draw_mode::to_gl(mode), first, count, instance_count)
        }
    }

    pub fn draw_elements_instanced(
        &self,
        mode: DrawMode,
        count: i32,
        element_type: u32,
        offset: i32,
        instance_count: i32,
    ) {
        unsafe {
            self.ctx.draw_elements_instanced(
                gl::draw_mode::to_gl(mode),
                count,
                element_type as u32,
                offset,
                instance_count,
            )
        }
    }

    pub fn set_viewport(&mut self, x: i32, y: i32, width: i32, height: i32) {
        let new_viewport = viewport::Viewport::new(x, y, width, height);
        if self.current_viewport != new_viewport {
            self.current_viewport = new_viewport;
            unsafe { self.ctx.viewport(x, y, width, height) }
        }
    }

    pub fn viewport(&self) -> viewport::Viewport<i32> {
        self.current_viewport
    }

    pub fn set_scissor(&mut self, region: Option<viewport::Viewport<i32>>) {
        match (region, &mut self.current_scissor) {
            (None, Some(_current)) => {
                unsafe {
                    self.ctx.disable(glow::SCISSOR_TEST);
                }
                self.current_scissor = None;
            }
            (Some(new), None) => {
                unsafe {
                    self.ctx.enable(glow::SCISSOR_TEST);
                    self.ctx
                        .scissor(new.x(), new.y(), new.width(), new.height());
                }
                self.current_scissor = Some(new);
            }
            (Some(new), Some(current)) => {
                if &new != current {
                    unsafe {
                        self.ctx
                            .scissor(new.x(), new.y(), new.width(), new.height());
                    }
                    *current = new;
                }
            }
            (None, None) => {}
        }
    }

    pub fn clear_color(&self, red: f32, green: f32, blue: f32, alpha: f32) {
        unsafe { self.ctx.clear_color(red, green, blue, alpha) }
    }

    pub fn clear(&self) {
        unsafe {
            self.ctx
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
        }
    }

    pub fn read_pixels(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: PixelFormat,
        data: &mut [u8],
    ) {
        let gl::TextureFormat { external, ty, .. } = gl::pixel_format::to_gl(format, &self.version);
        unsafe {
            self.ctx.read_pixels(
                x,
                y,
                width,
                height,
                external,
                ty,
                glow::PixelPackData::Slice(data),
            )
        }
    }

    pub fn debug_message_callback<F>(&self, mut callback: F)
    where
        F: FnMut(DebugSource, DebugType, u32, DebugSeverity, &str),
    {
        if self.ctx.supports_debug() {
            unsafe {
                self.ctx.enable(glow::DEBUG_OUTPUT);
                self.ctx
                    .debug_message_callback(|source, event_type, id, severity, msg| {
                        let source = match source {
                            glow::DEBUG_SOURCE_API => DebugSource::API,
                            glow::DEBUG_SOURCE_WINDOW_SYSTEM => DebugSource::WindowSystem,
                            glow::DEBUG_SOURCE_SHADER_COMPILER => DebugSource::ShaderCompiler,
                            glow::DEBUG_SOURCE_THIRD_PARTY => DebugSource::ThirdParty,
                            glow::DEBUG_SOURCE_APPLICATION => DebugSource::Application,
                            glow::DEBUG_SOURCE_OTHER => DebugSource::Other,
                            _ => DebugSource::Other,
                        };

                        let event_type = match event_type {
                            glow::DEBUG_TYPE_ERROR => DebugType::Error,
                            glow::DEBUG_TYPE_DEPRECATED_BEHAVIOR => DebugType::DeprecatedBehavior,
                            glow::DEBUG_TYPE_UNDEFINED_BEHAVIOR => DebugType::DeprecatedBehavior,
                            glow::DEBUG_TYPE_PORTABILITY => DebugType::Portability,
                            glow::DEBUG_TYPE_PERFORMANCE => DebugType::Performance,
                            glow::DEBUG_TYPE_MARKER => DebugType::Marker,
                            glow::DEBUG_TYPE_PUSH_GROUP => DebugType::PushGroup,
                            glow::DEBUG_TYPE_POP_GROUP => DebugType::PopGroup,
                            glow::DEBUG_TYPE_OTHER => DebugType::Other,
                            _ => DebugType::Other,
                        };

                        let severity = match severity {
                            glow::DEBUG_SEVERITY_HIGH => DebugSeverity::High,
                            glow::DEBUG_SEVERITY_MEDIUM => DebugSeverity::Medium,
                            glow::DEBUG_SEVERITY_LOW => DebugSeverity::Low,
                            glow::DEBUG_SEVERITY_NOTIFICATION => DebugSeverity::Notification,
                            _ => DebugSeverity::Notification,
                        };

                        callback(source, event_type, id, severity, msg)
                    });
            }
        }
    }
}

#[derive(Debug)]
pub enum DebugSeverity {
    High,
    Medium,
    Low,
    Notification,
}

#[derive(Debug)]
pub enum DebugType {
    Error,
    DeprecatedBehavior,
    UndefinedBehavior,
    Portability,
    Performance,
    Marker,
    PushGroup,
    PopGroup,
    Other,
}

#[derive(Debug)]
pub enum DebugSource {
    API,
    WindowSystem,
    ShaderCompiler,
    ThirdParty,
    Application,
    Other,
}

impl texture::TextureUpdate for Context {
    fn set_texture_sub_data(
        &mut self,
        texture_key: TextureKey,
        texture: texture::TextureInfo,
        texture_type: texture::TextureType,
        data: &[u8],
        x_offset: u32,
        y_offset: u32,
    ) {
        let gl::TextureFormat { external, ty, .. } =
            gl::pixel_format::to_gl(texture.get_format(), &self.version);
        let width = texture.width();
        let height = texture.height();
        let gl_target = gl::texture::to_gl(texture_type);
        self.bind_texture_to_unit(texture_type, texture_key, 0.into());
        unsafe {
            self.ctx.tex_sub_image_2d(
                gl_target,
                0,
                x_offset as i32,
                y_offset as i32,
                width as i32,
                height as i32,
                external,
                ty,
                glow::PixelUnpackData::Slice(data),
            );
            if texture.mipmaps() {
                self.ctx.generate_mipmap(gl_target);
            }
        }
    }

    fn set_texture_data(
        &mut self,
        texture_key: TextureKey,
        texture: texture::TextureInfo,
        texture_type: texture::TextureType,
        data: Option<&[u8]>,
    ) {
        let gl::TextureFormat {
            internal,
            external,
            ty,
            swizzle,
        } = gl::pixel_format::to_gl(texture.get_format(), &self.version);
        let width = texture.width();
        let height = texture.height();
        let gl_target = gl::texture::to_gl(texture_type);
        self.bind_texture_to_unit(texture_type, texture_key, 0.into());
        unsafe {
            if let Some(swizzle) = swizzle {
                self.ctx
                    .tex_parameter_i32(gl_target, glow::TEXTURE_SWIZZLE_R, swizzle[0]);
                self.ctx
                    .tex_parameter_i32(gl_target, glow::TEXTURE_SWIZZLE_G, swizzle[1]);
                self.ctx
                    .tex_parameter_i32(gl_target, glow::TEXTURE_SWIZZLE_B, swizzle[2]);
                self.ctx
                    .tex_parameter_i32(gl_target, glow::TEXTURE_SWIZZLE_A, swizzle[3]);
            }
            self.ctx.tex_image_2d(
                gl_target,
                0,
                internal as i32,
                width as i32,
                height as i32,
                0,
                external,
                ty,
                data,
            );
            if texture.mipmaps() {
                self.ctx.generate_mipmap(gl_target);
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn set_texture_data_with_html_image<T: texture::Texture>(
        &mut self,
        texture: T,
        data: &web_sys::HtmlImageElement,
    ) {
        let texture_info = texture.get_texture_info();
        let gl::TextureFormat {
            internal,
            external,
            ty,
            swizzle,
        } = gl::pixel_format::to_gl(texture_info.get_format(), &self.version);
        let gl_target = gl::texture::to_gl(texture.get_texture_type());
        self.bind_texture_to_unit(
            texture.get_texture_type(),
            texture.get_texture_key(),
            0.into(),
        );
        unsafe {
            if let Some(swizzle) = swizzle {
                self.ctx
                    .tex_parameter_i32(gl_target, glow::TEXTURE_SWIZZLE_R, swizzle[0]);
                self.ctx
                    .tex_parameter_i32(gl_target, glow::TEXTURE_SWIZZLE_G, swizzle[1]);
                self.ctx
                    .tex_parameter_i32(gl_target, glow::TEXTURE_SWIZZLE_B, swizzle[2]);
                self.ctx
                    .tex_parameter_i32(gl_target, glow::TEXTURE_SWIZZLE_A, swizzle[3]);
            }
            self.ctx.tex_image_2d_with_html_image(
                gl_target,
                0,
                internal as i32,
                external,
                ty,
                data,
            );
            if texture_info.mipmaps() {
                self.ctx.generate_mipmap(gl_target);
            }
        }
    }

    fn set_texture_wrap(
        &mut self,
        texture_key: TextureKey,
        texture_type: texture::TextureType,
        wrap: texture::Wrap,
    ) {
        let gl_target = gl::texture::to_gl(texture_type);
        unsafe {
            self.bind_texture_to_unit(texture_type, texture_key, 0.into());
            self.ctx.tex_parameter_i32(
                gl_target,
                glow::TEXTURE_WRAP_S,
                gl::wrap_mode::to_gl(wrap.s()) as i32,
            );
            self.ctx.tex_parameter_i32(
                gl_target,
                glow::TEXTURE_WRAP_T,
                gl::wrap_mode::to_gl(wrap.t()) as i32,
            );
            use texture::TextureType;
            match texture_type {
                TextureType::Tex2D | TextureType::Tex2DArray | TextureType::Cube => (),
                TextureType::Volume => self.ctx.tex_parameter_i32(
                    gl_target,
                    glow::TEXTURE_WRAP_R,
                    gl::wrap_mode::to_gl(wrap.r()) as i32,
                ),
            }
        }
    }

    fn set_texture_filter(
        &mut self,
        texture_key: TextureKey,
        texture_type: texture::TextureType,
        filter: texture::Filter,
    ) {
        use texture::FilterMode;

        let gl_min = match filter.min() {
            FilterMode::Nearest => glow::NEAREST,
            FilterMode::Linear | FilterMode::None => glow::LINEAR,
        };
        let gl_mag = match filter.mag() {
            FilterMode::Nearest => glow::NEAREST,
            FilterMode::Linear | FilterMode::None => glow::LINEAR,
        };

        let gl_min = match filter.mipmap() {
            FilterMode::None => gl_min,
            FilterMode::Nearest | FilterMode::Linear => match (filter.min(), filter.mipmap()) {
                (FilterMode::Nearest, FilterMode::Nearest) => glow::NEAREST_MIPMAP_NEAREST,
                (FilterMode::Nearest, FilterMode::Linear) => glow::NEAREST_MIPMAP_LINEAR,
                (FilterMode::Linear, FilterMode::Nearest) => glow::LINEAR_MIPMAP_NEAREST,
                (FilterMode::Linear, FilterMode::Linear) => glow::LINEAR_MIPMAP_LINEAR,
                _ => glow::LINEAR,
            },
        };

        let gl_target = gl::texture::to_gl(texture_type);
        unsafe {
            self.bind_texture_to_unit(texture_type, texture_key, 0.into());
            self.ctx
                .tex_parameter_i32(gl_target, glow::TEXTURE_MIN_FILTER, gl_min as i32);
            self.ctx
                .tex_parameter_i32(gl_target, glow::TEXTURE_MAG_FILTER, gl_mag as i32);
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        for (_, shader) in self.shaders.drain() {
            unsafe {
                self.ctx.delete_program(shader);
            }
        }

        for (_, buffer) in self.buffers.drain() {
            unsafe { self.ctx.delete_buffer(buffer) }
        }
    }
}

pub trait Renderer {
    fn clear(&mut self, settings: ClearSettings);
    fn draw<S, M>(&mut self, shader: &S, geometry: &Geometry<M>, settings: PipelineSettings)
    where
        S: shader::Shader + ?Sized,
        M: mesh::Mesh;
}

impl Renderer for Context {
    fn clear(&mut self, settings: ClearSettings) {
        let ClearSettings {
            color,
            depth,
            stencil,
            target,
            scissor,
        } = settings;
        self.set_scissor(scissor);

        let mut clear_bits = 0;

        if let Some(color) = color {
            let Color {
                red,
                blue,
                green,
                alpha,
            } = color.into();
            unsafe {
                self.ctx.clear_color(red, green, blue, alpha);
            }
            clear_bits |= glow::COLOR_BUFFER_BIT;
        }

        if let Some(depth) = depth {
            unsafe {
                self.ctx.clear_depth_f32(depth.0);
            }
            clear_bits |= glow::DEPTH_BUFFER_BIT;
        }

        if let Some(stencil) = stencil {
            unsafe {
                self.ctx.clear_stencil(stencil);
            }
            clear_bits |= glow::STENCIL_BUFFER_BIT;
        }

        self.bind_framebuffer(
            canvas::Target::All,
            target.map(canvas::Canvas::get_framebuffer_key),
        );
        unsafe {
            self.ctx.clear(clear_bits);
        }
    }

    fn draw<S, M>(&mut self, shader: &S, geometry: &Geometry<M>, settings: PipelineSettings)
    where
        S: shader::Shader + ?Sized,
        M: mesh::Mesh,
    {
        self.use_shader(Some(shader));

        if let Some(depth_state) = settings.depth_state {
            self.enable(Feature::DepthTest(depth_state.function));
        } else {
            self.disable(Feature::DepthTest(DepthFunction::Never));
        }
        self.set_scissor(settings.scissor_state);

        self.bind_framebuffer(
            canvas::Target::All,
            settings
                .framebuffer
                .map(canvas::Canvas::get_framebuffer_key),
        );

        let Geometry {
            mesh,
            draw_range,
            draw_mode,
            instance_count,
            ..
        } = geometry;

        let attached_attributes = mesh.attachments();
        let (desired_attribute_state, attributes) = prepare_draw(shader, &attached_attributes);
        self.set_vertex_attributes(desired_attribute_state, &attributes);

        mesh.draw(
            self,
            draw_range.clone(),
            *draw_mode,
            *instance_count as usize,
        );
    }
}

fn prepare_draw<'a, S: shader::Shader + ?Sized>(
    shader: &S,
    attached_attributes: &'a [mesh::AttachedAttributes],
) -> (u32, [Option<mesh::BindingInfo<'a>>; 32]) {
    // there's likely a better way to accumulate all bindings into an easy to search collection
    let attached_bindings = attached_attributes
        .iter()
        .flat_map(|attributes| {
            attributes
                .formats
                .iter()
                .map(|binding| {
                    (
                        binding,
                        attributes.stride,
                        attributes.step,
                        attributes.buffer.handle(),
                        attributes.buffer.buffer_type(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let mut desired_attribute_state = 0u32;
    let mut attributes = [None; 32];
    for attr in shader::Shader::attributes(shader).iter() {
        let binding = attached_bindings
            .iter()
            .find(|(binding, ..)| binding.name == attr.name.as_str())
            .cloned();
        if let Some(binding) = binding {
            desired_attribute_state |= 1 << attr.location;
            attributes[attr.location as usize] = Some(binding);
        }
    }
    (desired_attribute_state, attributes)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Geometry<M> {
    pub mesh: M,
    pub draw_range: std::ops::Range<usize>,
    pub draw_mode: DrawMode,
    pub instance_count: u32,
}

/// An non-NAN f32 value clamped between 0.0 and 1.0, inclusive.
/// This type can be constructed with an f32 which will be clamped into the appropriate range.
#[derive(Copy, Clone, Default, Debug, PartialOrd, PartialEq)]
pub struct ClampedF32(f32);

impl From<f32> for ClampedF32 {
    fn from(v: f32) -> Self {
        let v = if v < 0. {
            0f32
        } else if v > 1. {
            1.
        } else if v.is_nan() {
            0.
        } else {
            v
        };

        ClampedF32(v)
    }
}

impl Eq for ClampedF32 {}
impl Ord for ClampedF32 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).expect("infallible")
    }
}

impl std::ops::Deref for ClampedF32 {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct Color<T> {
    pub red: T,
    pub blue: T,
    pub green: T,
    pub alpha: T,
}

impl Into<Color<ClampedF32>> for Color<f32> {
    fn into(self) -> Color<ClampedF32> {
        Color::<ClampedF32> {
            red: self.red.into(),
            blue: self.blue.into(),
            green: self.green.into(),
            alpha: self.alpha.into(),
        }
    }
}

impl Into<Color<f32>> for Color<ClampedF32> {
    fn into(self) -> Color<f32> {
        Color::<f32> {
            red: self.red.0,
            blue: self.blue.0,
            green: self.green.0,
            alpha: self.alpha.0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ClearSettings<'a> {
    pub color: Option<Color<ClampedF32>>,
    pub depth: Option<ClampedF32>,
    pub stencil: Option<i32>, // TODO: does signed make sense here?
    pub target: Option<&'a canvas::Canvas>,
    pub scissor: Option<viewport::Viewport<i32>>,
}

impl Default for ClearSettings<'_> {
    fn default() -> Self {
        Self {
            color: Some(Color::default()),
            depth: Some(ClampedF32(1.)),
            stencil: Some(0),
            target: None,
            scissor: None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DepthState {
    pub function: DepthFunction,
    pub range: std::ops::RangeInclusive<ClampedF32>,
    pub write_mask: bool,
}

impl Default for DepthState {
    fn default() -> Self {
        Self {
            function: DepthFunction::Less,
            range: ClampedF32(0.)..=ClampedF32(1.),
            write_mask: true,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CullingState {
    pub mode: CullFace,
    pub winding: VertexWinding,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PolygonState {
    pub culling_state: Option<CullingState>,
    pub polygon_offset_units: f32,
    pub polygon_offset_factor: f32,
}

impl Default for PolygonState {
    fn default() -> Self {
        Self {
            culling_state: None,
            polygon_offset_units: 0.0,
            polygon_offset_factor: 0.0,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BlendSource {
    Zero,
    One,
    SourceColor,
    OneMinusSourceColor,
    DestinationColor,
    OneMinusDestinationColor,
    SourceAlpha,
    OneMinusSourceAlpha,
    DestinationAlpha,
    OneMinusDestinationAlpha,
    ConstantColor,
    OneMinusConstantColor,
    ConstantAlpha,
    OneMinusConstantAlpha,
    SourceAlphaSaturate,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BlendDestination {
    Zero,
    One,
    SourceColor,
    OneMinusSourceColor,
    DestinationColor,
    OneMinusDestinationColor,
    SourceAlpha,
    OneMinusSourceAlpha,
    DestinationAlpha,
    OneMinusDestinationAlpha,
    ConstantColor,
    OneMinusConstantColor,
    ConstantAlpha,
    OneMinusConstantAlpha,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BlendEquation {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BlendState {
    pub destination_rgb: BlendDestination,
    pub source_rgb: BlendSource,
    pub destination_alpha: BlendDestination,
    pub source_alpha: BlendSource,
    pub color: Color<ClampedF32>,
    pub equation_rgb: BlendEquation,
    pub equation_alpha: BlendEquation,
}

impl Default for BlendState {
    fn default() -> Self {
        Self {
            destination_rgb: BlendDestination::Zero,
            source_rgb: BlendSource::One,
            destination_alpha: BlendDestination::Zero,
            source_alpha: BlendSource::One,
            color: Default::default(),
            equation_rgb: BlendEquation::Add,
            equation_alpha: BlendEquation::Add,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StencilFunction {
    Never,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    Equal,
    NoteEqual,
    Always,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct StencilState {
    pub function: StencilFunction,
    // TODO the rest
}

impl Default for StencilState {
    fn default() -> Self {
        Self {
            function: StencilFunction::Always,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PipelineSettings<'a> {
    pub viewport: viewport::Viewport<i32>,
    pub framebuffer: Option<&'a canvas::Canvas>,
    pub polygon_state: PolygonState,
    pub depth_state: Option<DepthState>,
    pub blend_state: Option<BlendState>,
    pub stencil_state: Option<StencilState>,
    pub scissor_state: Option<viewport::Viewport<i32>>,
}

impl<'a> Default for PipelineSettings<'a> {
    fn default() -> Self {
        Self {
            viewport: Default::default(),
            framebuffer: None,
            depth_state: Some(DepthState::default()),
            polygon_state: Default::default(),
            blend_state: None,
            stencil_state: None,
            scissor_state: None,
        }
    }
}

#[cfg(all(test, not(target_os = "linux")))]
mod tests {
    use super::*;

    #[test]
    fn pipeline() {
        let pipeline_settings = PipelineSettings::default();
        println!("{:#?}", pipeline_settings);

        let pipeline_settings = PipelineSettings {
            viewport: viewport::Viewport::new(0, 0, 720, 480),
            ..PipelineSettings::default()
        };
        println!("{:#?}", pipeline_settings);

        let pipeline_settings = PipelineSettings {
            depth_state: Some(DepthState {
                function: DepthFunction::Never,
                ..DepthState::default()
            }),
            ..pipeline_settings
        };
        println!("{:#?}", pipeline_settings);
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Default, bytemuck::Zeroable, bytemuck::Pod)]
    struct TestVertex {
        color: f32,
        position: f32,
    }

    use vertex::VertexFormat;
    impl vertex::Vertex for TestVertex {
        fn build_bindings() -> &'static [VertexFormat] {
            &[
                VertexFormat {
                    name: "color",
                    offset: 0,
                    atype: vertex::AttributeType::F32,
                    normalize: false,
                },
                VertexFormat {
                    name: "position",
                    offset: std::mem::size_of::<f32>(),
                    atype: vertex::AttributeType::F32,
                    normalize: false,
                },
            ]
        }
    }

    fn get_headless_context(
        width: u32,
        height: u32,
    ) -> (glow::Context, glutin::Context<glutin::PossiblyCurrent>) {
        use glutin::platform::windows::EventLoopExtWindows;
        let el = glutin::event_loop::EventLoop::<()>::new_any_thread();
        let window = glutin::ContextBuilder::new()
            .build_headless(&el, glutin::dpi::PhysicalSize::new(width, height))
            .unwrap();
        let window = unsafe { window.make_current().unwrap() };
        (
            unsafe { glow::Context::from_loader_function(|name| window.get_proc_address(name)) },
            window,
        )
    }

    #[test]
    fn basic() {
        let (ctx, _window) = get_headless_context(100, 100);
        let ctx = Context::new(ctx);
        ctx.clear();
    }

    #[test]
    fn unused_vertex_attribute() {
        let (ctx, _window) = get_headless_context(100, 100);
        let mut ctx = Context::new(ctx);

        let mesh = mesh::VertexMesh::with_data(
            &mut ctx,
            &[
                TestVertex {
                    color: 0.,
                    position: 1.,
                },
                TestVertex {
                    color: 1.,
                    position: 2.,
                },
                TestVertex {
                    color: 2.,
                    position: 3.,
                },
            ],
        )
        .unwrap();

        const SRC: &str = r#"
varying vec4 vColor;

#ifdef VERTEX
layout(location = 2) attribute vec4 position;

void main() {
    gl_Position = position;
}
#endif

#ifdef FRAGMENT
void main() {
    fragColor = vec4(1., 1., 1., 1.);
}
#endif"#;

        let (vert, frag) = shader::DynamicShader::create_source(SRC, SRC);
        let shader = shader::DynamicShader::new(&mut ctx, &vert, &frag).unwrap();
        ctx.use_shader(Some(&shader));

        Renderer::draw(
            &mut ctx,
            &shader,
            &super::Geometry {
                mesh: &mesh,
                draw_range: 0..1,
                draw_mode: DrawMode::Triangles,
                instance_count: 1,
            },
            PipelineSettings::default(),
        );
    }

    #[test]
    fn mapped_mesh() {
        let (ctx, _window) = get_headless_context(100, 100);
        let mut ctx = Context::new(ctx);

        let vertices = [
            TestVertex {
                color: 0.,
                position: 1.,
            },
            TestVertex {
                color: 1.,
                position: 2.,
            },
            TestVertex {
                color: 2.,
                position: 3.,
            },
        ];

        let indices = [0u32, 1, 2];

        {
            let mut mesh = mesh::MappedVertexMesh::new(&mut ctx, 3).unwrap();
            mesh.set_vertices(&vertices, 0);

            let mapped_verts = mesh.get_vertices();
            assert_eq!(vertices, mapped_verts);
        }

        {
            let mut mesh = mesh::MappedIndexedMesh::new(&mut ctx, 3, 3).unwrap();
            mesh.set_vertices(&vertices, 0);
            mesh.set_indices(&indices, 0);

            assert_eq!(vertices, mesh.get_vertices());
            assert_eq!(indices, mesh.get_indices());
        }
    }

    #[test]
    fn mapped_image() {
        use super::PixelFormat;
        use image::*;
        use texture::*;

        let (ctx, _window) = get_headless_context(100, 100);
        let mut ctx = Context::new(ctx);
        {
            // RGBA
            let data = vec![234; 3 * 3 * 4];

            let mut image = MappedImage::with_data(
                &mut ctx,
                TextureType::Tex2D,
                PixelFormat::RGBA8,
                3,
                3,
                data.clone(),
                Settings::default(),
            )
            .unwrap();
            let pixel_stride = image.pixel_stride();
            assert_eq!(image.get_pixels(), data);

            let pixel = [1, 2, 3, 4];
            image.set_pixels(viewport::Viewport::new(0, 0, 1, 1), &pixel);
            assert_eq!(image.get_pixels()[..4], pixel);

            let pixel = [0, 0, 1, 0];
            image.set_pixel(0, 0, &pixel);
            assert_eq!(image.get_pixels()[..4], pixel);
            assert_eq!(image.get_pixel(0, 0), pixel);

            assert_eq!(image.get_pixel(0, 1), [234, 234, 234, 234]);
            image.set_pixel(0, 1, &pixel);
            assert_eq!(image.get_pixel(0, 1), pixel);
            assert_eq!(
                image.get_pixels()[(3 * pixel_stride)..(4 * pixel_stride)],
                pixel
            );
        }

        {
            // RGB
            let data = vec![234; 3 * 3 * 3];

            let mut image = MappedImage::with_data(
                &mut ctx,
                TextureType::Tex2D,
                PixelFormat::RGB8,
                3,
                3,
                data.clone(),
                Settings::default(),
            )
            .unwrap();
            let pixel_stride = image.pixel_stride();
            assert_eq!(image.get_pixels(), data);

            let pixel = [1, 2, 3];
            image.set_pixels(viewport::Viewport::new(0, 0, 1, 1), &pixel);
            assert_eq!(image.get_pixels()[..pixel_stride], pixel);

            let pixel = [0, 0, 1];
            image.set_pixel(0, 0, &pixel);
            assert_eq!(image.get_pixels()[..pixel_stride], pixel);
            assert_eq!(image.get_pixel(0, 0), pixel);

            assert_eq!(image.get_pixel(0, 1), [234, 234, 234]);
            image.set_pixel(0, 1, &pixel);
            assert_eq!(image.get_pixel(0, 1), pixel);
            assert_eq!(
                image.get_pixels()[(3 * pixel_stride)..(4 * pixel_stride)],
                pixel
            );
        }
    }

    #[test]
    fn quad_batch_test() {
        let (ctx, _window) = get_headless_context(100, 100);
        let mut ctx = Context::new(ctx);

        let quad = quad_batch::Quad::from(viewport::Viewport::new(0., 0., 1., 1.)).map(|(x, y)| {
            TestVertex {
                color: y,
                position: x,
            }
        });
        let mut batch = quad_batch::QuadBatch::<TestVertex>::new(&mut ctx, 1).unwrap();
        let index = batch.push(quad.clone());

        assert_eq!(batch.get_quad(index).unwrap(), quad);
    }
}
