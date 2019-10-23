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

use glow::HasContext;
use slotmap::DenseSlotMap;
use std::collections::{hash_map::Entry, HashMap};

type GLBuffer = <glow::Context as HasContext>::Buffer;
type GLProgram = <glow::Context as HasContext>::Program;
type GLTexture = <glow::Context as HasContext>::Texture;
type GLFrameBuffer = <glow::Context as HasContext>::Framebuffer;

slotmap::new_key_type! {
    pub struct ShaderKey;
    pub struct BufferKey;
    pub struct TextureKey;
    pub struct FramebufferKey;
}

pub struct DebugGroup<'a> {
    ctx: &'a glow::Context,
}

impl<'a> DebugGroup<'a> {
    pub fn new(ctx: &'a glow::Context, message: &str) -> Self {
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

fn to_index(target: canvas::Target) -> usize {
    match target {
        Target::Draw => 0,
        Target::Read => 1,
        Target::All => 0,
    }
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

pub enum Feature {
    DepthTest(DepthFunction),
}

struct GLConstants {
    max_vertex_attributes: usize,
    max_texture_units: usize,
}

// a caching, convenience and safety layer around glow
pub struct Context {
    ctx: glow::Context,
    gl_constants: GLConstants,
    shaders: DenseSlotMap<ShaderKey, shader::Shader>,
    active_shader: Option<ShaderKey>,
    buffers: DenseSlotMap<BufferKey, buffer::Buffer>,
    active_buffers: HashMap<buffer::BufferType, BufferKey>,
    textures: DenseSlotMap<TextureKey, GLTexture>,
    bound_textures: Vec<Vec<Option<GLTexture>>>,
    framebuffers: DenseSlotMap<FramebufferKey, GLFrameBuffer>,
    active_framebuffer: [Option<FramebufferKey>; 2],
    current_texture_unit: u32,
    current_viewport: viewport::Viewport<i32>,
    enabled_attributes: u32, // a bitmask that represents the vertex attribute state
}

impl Context {
    pub fn new(ctx: glow::Context) -> Self {
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
            .map(|tt| vec![None; gl_constants.max_texture_units])
            .collect();

        for texture_unit in 0..gl_constants.max_texture_units {
            unsafe {
                ctx.active_texture(glow::TEXTURE0 + texture_unit as u32);
                // do this for every supported texture type
                ctx.bind_texture(glow::TEXTURE_2D, None);
            }
        }
        unsafe { ctx.active_texture(glow::TEXTURE0) }
        unsafe {
            // TODO: this should be left to the consumer
            ctx.enable(glow::CULL_FACE);
            ctx.cull_face(glow::BACK);
            ctx.enable(glow::BLEND);
            ctx.blend_equation(glow::FUNC_ADD);
            ctx.blend_func_separate(
                glow::SRC_ALPHA,
                glow::ONE_MINUS_SRC_ALPHA,
                glow::ONE,
                glow::ONE_MINUS_SRC_ALPHA,
            );
        }
        let mut ctx = Self {
            ctx,
            gl_constants,
            shaders: DenseSlotMap::with_key(),
            active_shader: None,
            buffers: DenseSlotMap::with_key(),
            active_buffers: HashMap::new(),
            textures: DenseSlotMap::with_key(),
            bound_textures,
            framebuffers: DenseSlotMap::with_key(),
            active_framebuffer: [None; 2],
            current_texture_unit: 0u32,
            current_viewport: viewport::Viewport::default(),
            enabled_attributes: std::u32::MAX,
        };
        ctx.set_vertex_attributes(0, 0, &vec![]);
        ctx
    }

    pub fn enable(&mut self, feature: Feature) {
        match feature {
            Feature::DepthTest(func) => unsafe {
                self.ctx.enable(glow::DEPTH_TEST);
                self.ctx.depth_func(func.to_gl());
            },
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
    ) -> BufferKey {
        // the implementation of Buffer::new leaks here in that we bind the buffer after we
        // create it so it's tracked in the active buffers by necessity
        let buffer = buffer::Buffer::new(&self.ctx, size, buffer_type, usage);
        let buffer_key = self.buffers.insert(buffer);
        self.active_buffers.insert(buffer_type, buffer_key);
        buffer_key
    }

    pub fn destroy_buffer(&mut self, buffer: BufferKey) {
        self.buffers.remove(buffer).map(|buffer| unsafe {
            self.ctx.delete_buffer(buffer.handle());
        });
    }

    pub fn bind_buffer(&mut self, buffer_key: BufferKey) {
        match self.buffers.get(buffer_key) {
            None => (),
            Some(buffer) => match self.active_buffers.entry(buffer.buffer_type()) {
                Entry::Occupied(mut o) => {
                    if *o.get() != buffer_key {
                        o.insert(buffer_key);
                        unsafe {
                            self.ctx
                                .bind_buffer(buffer.buffer_type().into(), Some(buffer.handle()))
                        }
                    }
                }
                Entry::Vacant(v) => {
                    v.insert(buffer_key);
                    unsafe {
                        self.ctx
                            .bind_buffer(buffer.buffer_type().into(), Some(buffer.handle()))
                    }
                }
            },
        }
    }

    pub fn unmap_buffer(&mut self, buffer_key: BufferKey) {
        self.bind_buffer(buffer_key);
        self.get_buffer(buffer_key).map(|buffer| {
            if buffer.modified_size() > 0 {
                let target = buffer.buffer_type().into();
                let data = buffer.memory_map();
                let usage = buffer.usage().into();
                unsafe {
                    self.ctx.buffer_data_u8_slice(target, data, usage);
                }
            }
        });
        self.get_buffer_mut(buffer_key)
            .map(|buffer| buffer.reset_modified_range());
    }

    pub fn get_buffer(&self, buffer: BufferKey) -> Option<&buffer::Buffer> {
        self.buffers.get(buffer)
    }

    pub fn get_buffer_mut(&mut self, buffer: BufferKey) -> Option<&mut buffer::Buffer> {
        self.buffers.get_mut(buffer)
    }

    pub fn new_shader(&mut self, vertex_source: &str, fragment_source: &str) -> ShaderKey {
        let shader = shader::Shader::new(&self.ctx, vertex_source, fragment_source).unwrap();
        self.shaders.insert(shader)
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

    pub fn new_texture(&mut self, texture_type: texture::TextureType) -> TextureKey {
        unsafe {
            let handle = self.ctx.create_texture().unwrap();
            let texture = self.textures.insert(handle);
            self.ctx.active_texture(glow::TEXTURE0);
            //            self.ctx.bind_texture(texture_type.to_gl(), Some(handle));
            self.bind_texture_to_unit(texture_type, texture, 0);
            texture
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
        texture_unit: u32,
    ) {
        let texture_unit_index = texture_unit as usize;
        match (
            self.textures.get(texture_key),
            self.bound_textures[texture_type.to_index()][texture_unit_index],
        ) {
            (Some(&texture), None) => {
                self.bound_textures[texture_type.to_index()][texture_unit_index] = Some(texture);
                unsafe { self.ctx.bind_texture(texture_type.to_gl(), Some(texture)) }
            }
            (Some(&texture), Some(bound_texture)) => {
                if texture != bound_texture {
                    self.bound_textures[texture_type.to_index()][texture_unit_index] =
                        Some(texture);
                    unsafe { self.ctx.bind_texture(texture_type.to_gl(), Some(texture)) }
                }
            }
            (None, Some(_)) => {
                self.bound_textures[texture_type.to_index()][texture_unit_index] = None;
                unsafe { self.ctx.bind_texture(texture_type.to_gl(), None) }
            }
            (None, None) => (),
        }
    }

    pub fn set_texture_data(
        &mut self,
        texture_key: TextureKey,
        texture: texture::Texture,
        texture_type: texture::TextureType,
        data: Option<&[u8]>,
    ) {
        self.new_debug_group("Buffer Image Data");
        let (_internal, external, gl_type) = texture.format().to_gl();
        let width = texture.width();
        let height = texture.height();
        let gl_target = texture_type.to_gl();
        self.bind_texture_to_unit(texture_type, texture_key, 0);
        unsafe {
            self.ctx.tex_image_2d(
                gl_target,
                0,
                external as i32,
                width as i32,
                height as i32,
                0,
                external,
                gl_type,
                data,
            );
            self.ctx.generate_mipmap(gl_target);
        }
    }

    pub fn set_texture_wrap(&mut self, texture_key: TextureKey, texture_type: texture::TextureType, wrap: texture::Wrap) {
        use texture::TextureType;

        let gl_target = texture_type.to_gl();
        unsafe {
            self.bind_texture_to_unit(texture_type, texture_key, 0);
            self.ctx
                .tex_parameter_i32(gl_target, glow::TEXTURE_WRAP_S, wrap.s().to_gl() as i32);
            self.ctx
                .tex_parameter_i32(gl_target, glow::TEXTURE_WRAP_T, wrap.t().to_gl() as i32);
            match texture_type {
                TextureType::Tex2D | TextureType::Tex2DArray | TextureType::Cube => (),
                TextureType::Volume => self.ctx.tex_parameter_i32(
                    gl_target,
                    glow::TEXTURE_WRAP_R,
                    wrap.r().to_gl() as i32,
                ),
            }
        }
    }

    pub fn set_texture_filter(&mut self, texture_key: TextureKey, texture_type: texture::TextureType, filter: texture::Filter) {
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

        let gl_target = texture_type.to_gl();
        unsafe {
            self.bind_texture_to_unit(texture_type, texture_key, 0);
            self.ctx
                .tex_parameter_i32(gl_target, glow::TEXTURE_MIN_FILTER, gl_min as i32);
            self.ctx
                .tex_parameter_i32(gl_target, glow::TEXTURE_MAG_FILTER, gl_mag as i32);
        }
    }

    pub fn new_framebuffer(&mut self) -> FramebufferKey {
        let framebuffer = unsafe { self.ctx.create_framebuffer().unwrap() };
        self.framebuffers.insert(framebuffer)
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
        let target_index = to_index(target);
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
        self.active_framebuffer[to_index(target)]
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
            self.ctx.framebuffer_texture(
                target.to_gl(),
                attachment.to_gl(),
                self.textures.get(texture_key).map(|t| *t),
                level as i32,
            )
        }
    }

    pub fn set_vertex_attributes(
        &mut self,
        desired: u32,
        stride: usize,
        stuff: &Vec<(&vertex::VertexFormat, BufferKey)>,
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
                let (binding, buffer_key) = stuff[i as usize];
                self.bind_buffer(buffer_key);
                let (data_type, elements_count, instances_count) = binding.atype.to_gl();
                unsafe {
                    self.ctx.vertex_attrib_pointer_f32(
                        i,
                        elements_count,
                        data_type,
                        binding.normalize,
                        stride as i32,
                        binding.offset as i32,
                    );
                }
            }
        }

        self.enabled_attributes = desired;
    }

    pub fn set_uniform(&self, uniform_name: &str, data: &shader::RawUniformValue) {
        match self.active_shader {
            None => (),
            Some(active_shader) => match self.shaders.get(active_shader) {
                None => (),
                Some(shader) => match shader.get_uniform_by_name(uniform_name) {
                    None => (),
                    Some(uniform) => {
                        use shader::RawUniformValue;
                        let location = Some(uniform.location);
                        unsafe {
                            match data {
                                RawUniformValue::SignedInt(data) => {
                                    self.ctx.uniform_1_i32(location, *data)
                                }
                                RawUniformValue::Float(data) => {
                                    self.ctx.uniform_1_f32(location, *data)
                                }
                                RawUniformValue::Mat2(data) => {
                                    self.ctx.uniform_matrix_2_f32_slice(location, false, data)
                                }
                                RawUniformValue::Mat3(data) => {
                                    self.ctx.uniform_matrix_3_f32_slice(location, false, data)
                                }
                                RawUniformValue::Mat4(data) => {
                                    self.ctx.uniform_matrix_4_f32_slice(location, false, data)
                                }
                                RawUniformValue::Vec2(data) => {
                                    self.ctx.uniform_2_f32_slice(location, data)
                                }
                                RawUniformValue::Vec3(data) => {
                                    self.ctx.uniform_3_f32_slice(location, data)
                                }
                                RawUniformValue::Vec4(data) => {
                                    self.ctx.uniform_4_f32_slice(location, data)
                                }
                                RawUniformValue::IntVec2(data) => {
                                    self.ctx.uniform_2_i32_slice(location, data)
                                }
                                RawUniformValue::IntVec3(data) => {
                                    self.ctx.uniform_3_i32_slice(location, data)
                                }
                                RawUniformValue::IntVec4(data) => {
                                    self.ctx.uniform_4_i32_slice(location, data)
                                }
                            }
                        }
                    }
                },
            },
        }
    }

    pub fn draw_arrays(&self, mode: u32, first: i32, count: i32) {
        unsafe {
            self.ctx.draw_arrays(mode, first, count);
        }
    }

    pub fn draw_elements(&self, mode: u32, count: i32, element_type: u32, offset: i32) {
        unsafe {
            self.ctx.draw_elements(mode, count, element_type, offset);
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

impl Drop for Context {
    fn drop(&mut self) {
        for shader in self.shaders.values() {
            unsafe {
                self.ctx.delete_program(shader.handle());
            }
        }

        for buffer in self.buffers.values() {
            unsafe { self.ctx.delete_buffer(buffer.handle()) }
        }
    }
}
