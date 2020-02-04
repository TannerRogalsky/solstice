pub extern crate glow;
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
use slotmap::{DenseSlotMap, SlotMap};
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
}

#[cfg(not(test))]
type GLContext = glow::Context;
#[cfg(test)]
type GLContext = gl::null_context::NullContext;

type GLBuffer = <GLContext as HasContext>::Buffer;
type GLProgram = <GLContext as HasContext>::Program;
type GLTexture = <GLContext as HasContext>::Texture;
type GLFrameBuffer = <GLContext as HasContext>::Framebuffer;
type GLUniformLocation = <GLContext as HasContext>::UniformLocation;

slotmap::new_key_type! {
    pub struct ShaderKey;
    pub struct BufferKey;
    pub struct TextureKey;
    pub struct FramebufferKey;
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

pub enum VertexWinding {
    ClockWise,
    CounterClockWise,
}

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

#[derive(Copy, Clone, Debug)]
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
pub struct TextureUnit(u32);

impl From<u32> for TextureUnit {
    fn from(v: u32) -> Self {
        TextureUnit(v)
    }
}

impl From<i32> for TextureUnit {
    fn from(v: i32) -> Self {
        TextureUnit(v as u32)
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
            (s.chars().nth(0), s.chars().nth(2), false, false)
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
    shaders: DenseSlotMap<ShaderKey, shader::Shader>, // TODO: evaluate the correctness of this. all other tracking is on GL primitives
    active_shader: Option<ShaderKey>,
    buffers: SlotMap<BufferKey, GLBuffer>,
    active_buffers: [Option<BufferKey>; 2],
    textures: SlotMap<TextureKey, GLTexture>,
    bound_textures: Vec<Vec<Option<GLTexture>>>,
    framebuffers: SlotMap<FramebufferKey, GLFrameBuffer>,
    active_framebuffer: [Option<FramebufferKey>; 2],
    current_texture_unit: TextureUnit,
    current_viewport: viewport::Viewport<i32>,
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
            shaders: DenseSlotMap::with_key(),
            active_shader: None,
            buffers: SlotMap::with_key(),
            active_buffers: [None; 2],
            textures: SlotMap::with_key(),
            bound_textures,
            framebuffers: SlotMap::with_key(),
            active_framebuffer: [None; 2],
            current_texture_unit: TextureUnit(0),
            current_viewport: viewport::Viewport::default(),
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
    ) -> Result<BufferKey, GraphicsError> {
        let vbo = unsafe {
            let vbo = self
                .ctx
                .create_buffer()
                .map_err(|_| GraphicsError::BufferError)?;
            self.ctx.bind_buffer(buffer_type.into(), Some(vbo));
            self.ctx
                .buffer_data_size(buffer_type.into(), size as _, usage.to_gl());
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

    fn buffer_static_draw(
        &self,
        buffer: &buffer::Buffer,
        modified_offset: usize,
        modified_size: usize,
    ) {
        let target = buffer.buffer_type().into();
        let data = &buffer.memory_map()[modified_offset..(modified_offset + modified_size)];
        unsafe {
            self.ctx
                .buffer_sub_data_u8_slice(target, modified_offset as i32, data)
        }
    }

    fn buffer_stream_draw(&self, buffer: &buffer::Buffer) {
        let target = buffer.buffer_type().into();
        let data = buffer.memory_map();

        unsafe {
            // "orphan" current buffer to avoid implicit synchronisation on the GPU:
            // http://www.seas.upenn.edu/~pcozzi/OpenGLInsights/OpenGLInsights-AsynchronousBufferTransfers.pdf
            self.ctx
                .buffer_data_size(target, buffer.size() as i32, buffer.usage().to_gl());
            self.ctx.buffer_sub_data_u8_slice(target, 0, data);
        }
    }

    pub fn unmap_buffer(&mut self, buffer: &mut buffer::Buffer) {
        self.bind_buffer(buffer.handle(), buffer.buffer_type());
        if self.buffers.get(buffer.handle()).is_some() {
            let modified_offset = std::cmp::min(buffer.modified_offset(), buffer.size() - 1);
            let modified_size =
                std::cmp::min(buffer.modified_size(), buffer.size() - modified_offset);

            if buffer.modified_size() > 0 {
                match buffer.usage() {
                    buffer::Usage::Stream => self.buffer_stream_draw(buffer),
                    buffer::Usage::Static => {
                        self.buffer_static_draw(buffer, modified_offset, modified_size)
                    }
                    buffer::Usage::Dynamic => {
                        if modified_size >= buffer.size() / 3 {
                            self.buffer_stream_draw(buffer);
                        } else {
                            self.buffer_static_draw(buffer, modified_offset, modified_size);
                        }
                    }
                }
            }
            buffer.reset_modified_range();
        }
    }

    pub fn new_shader(
        &mut self,
        vertex_source: &str,
        fragment_source: &str,
    ) -> Result<ShaderKey, GraphicsError> {
        shader::Shader::new(&self.ctx, vertex_source, fragment_source)
            .map(|shader| self.shaders.insert(shader))
            .map_err(GraphicsError::ShaderError)
    }

    pub fn destroy_shader(&mut self, shader: ShaderKey) {
        match self.shaders.remove(shader) {
            None => (),
            Some(shader) => unsafe {
                self.ctx.delete_program(shader.handle());
            },
        }
    }

    pub fn use_shader(&mut self, shader: Option<ShaderKey>) {
        if self.active_shader != shader {
            match &shader {
                None => {
                    self.active_shader = shader;
                    unsafe { self.ctx.use_program(None) }
                }
                Some(shader_key) => match self.shaders.get(*shader_key) {
                    None => (), // todo: define behaviour for `use_shader` with non-existent shader id
                    Some(actual_shader) => {
                        self.active_shader = shader;
                        unsafe { self.ctx.use_program(Some(actual_shader.handle())) }
                    }
                },
            }
        }
    }

    pub fn get_shader(&self, shader: ShaderKey) -> Option<&shader::Shader> {
        self.shaders.get(shader)
    }

    pub fn get_active_shader(&self) -> Option<ShaderKey> {
        self.active_shader
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
            self.bind_texture_to_unit(texture_type, texture, TextureUnit(0));
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
        let texture_unit_index = texture_unit.0 as usize;
        match (
            self.textures.get(texture_key),
            self.bound_textures[texture_type.to_index()][texture_unit_index],
        ) {
            (Some(&texture), None) => {
                if texture_unit != self.current_texture_unit {
                    unsafe {
                        self.ctx.active_texture(texture_unit.0);
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
                            self.ctx.active_texture(texture_unit.0);
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
                        self.ctx.active_texture(texture_unit.0);
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

    pub fn set_vertex_attributes(
        &mut self,
        desired: u32,
        stuff: &[(
            &vertex::VertexFormat,
            usize,
            u32,
            BufferKey,
            buffer::BufferType,
        )],
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
                let (vertex_format, stride, step, buffer_key, buffer_type) = stuff[i as usize];
                self.bind_buffer(buffer_key, buffer_type);
                let (data_type, elements_count, instances_count) = vertex_format.atype.to_gl();
                unsafe {
                    self.ctx.vertex_attrib_divisor(i, step);
                    self.ctx.vertex_attrib_pointer_f32(
                        i,
                        elements_count,
                        data_type,
                        vertex_format.normalize,
                        stride as i32,
                        vertex_format.offset as i32,
                    );
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
        format: data::PixelFormat,
        data: &mut [u8],
    ) {
        let (_, format, gl_type) = gl::pixel_format::to_gl(format, &self.version);
        unsafe {
            self.ctx
                .read_pixels(x, y, width, height, format, gl_type, data)
        }
    }

    pub fn debug_message_callback<F>(&self, callback: F)
    where
        F: FnMut(u32, u32, u32, u32, &str),
    {
        if self.ctx.supports_debug() {
            unsafe {
                self.ctx.enable(glow::DEBUG_OUTPUT);
                self.ctx.debug_message_callback(callback);
            }
        }
    }
}

impl texture::TextureUpdate for Context {
    fn set_texture_sub_data(
        &mut self,
        texture_key: TextureKey,
        texture: texture::TextureInfo,
        texture_type: texture::TextureType,
        data: Option<&[u8]>,
        x_offset: u32,
        y_offset: u32,
    ) {
        let (_internal, external, gl_type) =
            gl::pixel_format::to_gl(texture.get_format(), &self.version);
        let width = texture.width();
        let height = texture.height();
        let gl_target = gl::texture::to_gl(texture_type);
        self.bind_texture_to_unit(texture_type, texture_key, TextureUnit(0));
        unsafe {
            self.ctx.tex_sub_image_2d_u8_slice(
                gl_target,
                0,
                x_offset as i32,
                y_offset as i32,
                width as i32,
                height as i32,
                external,
                gl_type,
                data,
            );
        }
    }

    fn set_texture_data(
        &mut self,
        texture_key: TextureKey,
        texture: texture::TextureInfo,
        texture_type: texture::TextureType,
        data: Option<&[u8]>,
    ) {
        self.new_debug_group("Buffer Image Data");
        let (internal, external, gl_type) =
            gl::pixel_format::to_gl(texture.get_format(), &self.version);
        let width = texture.width();
        let height = texture.height();
        let gl_target = gl::texture::to_gl(texture_type);
        self.bind_texture_to_unit(texture_type, texture_key, TextureUnit(0));
        unsafe {
            self.ctx.tex_image_2d(
                gl_target,
                0,
                internal as i32,
                width as i32,
                height as i32,
                0,
                external,
                gl_type,
                data,
            );
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
            self.bind_texture_to_unit(texture_type, texture_key, TextureUnit(0));
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
            self.bind_texture_to_unit(texture_type, texture_key, TextureUnit(0));
            self.ctx
                .tex_parameter_i32(gl_target, glow::TEXTURE_MIN_FILTER, gl_min as i32);
            self.ctx
                .tex_parameter_i32(gl_target, glow::TEXTURE_MAG_FILTER, gl_mag as i32);
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        for shader in self.shaders.values() {
            unsafe {
                self.ctx.delete_program(shader.handle());
            }
        }

        for (_, buffer) in self.buffers.drain() {
            unsafe { self.ctx.delete_buffer(buffer) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let ctx = Context::new(GLContext {});
        ctx.clear();
    }
}
