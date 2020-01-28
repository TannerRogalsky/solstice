use glow::{ActiveAttribute, ActiveUniform, DebugMessageLogEntry, HasContext};

pub struct NullContext {}

impl HasContext for NullContext {
    type Shader = ();
    type Program = ();
    type Buffer = ();
    type VertexArray = ();
    type Texture = ();
    type Sampler = ();
    type Fence = ();
    type Framebuffer = ();
    type Renderbuffer = ();
    type UniformLocation = ();

    fn supports_debug(&self) -> bool {
        false
    }

    unsafe fn create_framebuffer(&self) -> Result<Self::Framebuffer, String> {
        Ok(())
    }

    unsafe fn create_renderbuffer(&self) -> Result<Self::Renderbuffer, String> {
        Ok(())
    }

    unsafe fn create_sampler(&self) -> Result<Self::Sampler, String> {
        Ok(())
    }

    unsafe fn create_shader(&self, _shader_type: u32) -> Result<Self::Shader, String> {
        Ok(())
    }

    unsafe fn create_texture(&self) -> Result<Self::Texture, String> {
        Ok(())
    }

    unsafe fn delete_shader(&self, _shader: Self::Shader) {}

    unsafe fn shader_source(&self, _shader: Self::Shader, _source: &str) {}

    unsafe fn compile_shader(&self, _shader: Self::Shader) {}

    unsafe fn get_shader_compile_status(&self, _shader: Self::Shader) -> bool {
        false
    }

    unsafe fn get_shader_info_log(&self, _shader: Self::Shader) -> String {
        String::new()
    }

    unsafe fn get_tex_image_u8_slice(
        &self,
        _target: u32,
        _level: i32,
        _format: u32,
        _ty: u32,
        _pixels: Option<&[u8]>,
    ) {
    }

    unsafe fn get_tex_image_pixel_buffer_offset(
        &self,
        _target: u32,
        _level: i32,
        _format: u32,
        _ty: u32,
        _pixel_buffer_offset: i32,
    ) {
    }

    unsafe fn create_program(&self) -> Result<Self::Program, String> {
        Ok(())
    }

    unsafe fn delete_program(&self, _program: Self::Program) {}

    unsafe fn attach_shader(&self, _program: Self::Program, _shader: Self::Shader) {}

    unsafe fn detach_shader(&self, _program: Self::Program, _shader: Self::Shader) {}

    unsafe fn link_program(&self, _program: Self::Program) {}

    unsafe fn get_program_link_status(&self, _program: Self::Program) -> bool {
        false
    }

    unsafe fn get_program_info_log(&self, _program: Self::Program) -> String {
        String::new()
    }

    unsafe fn get_active_uniforms(&self, _program: Self::Program) -> u32 {
        0
    }

    unsafe fn get_active_uniform(
        &self,
        _program: Self::Program,
        _index: u32,
    ) -> Option<ActiveUniform> {
        None
    }

    unsafe fn use_program(&self, _program: Option<Self::Program>) {}

    unsafe fn create_buffer(&self) -> Result<Self::Buffer, String> {
        Ok(())
    }

    unsafe fn bind_buffer(&self, _target: u32, _buffer: Option<Self::Buffer>) {}

    unsafe fn bind_buffer_range(
        &self,
        _target: u32,
        _index: u32,
        _buffer: Option<Self::Buffer>,
        _offset: i32,
        _size: i32,
    ) {
    }

    unsafe fn bind_framebuffer(&self, _target: u32, _framebuffer: Option<Self::Framebuffer>) {}

    unsafe fn bind_renderbuffer(&self, _target: u32, _renderbuffer: Option<Self::Renderbuffer>) {}

    unsafe fn blit_framebuffer(
        &self,
        _src_x0: i32,
        _src_y0: i32,
        _src_x1: i32,
        _src_y1: i32,
        _dst_x0: i32,
        _dst_y0: i32,
        _dst_x1: i32,
        _dst_y1: i32,
        _mask: u32,
        _filter: u32,
    ) {
    }

    unsafe fn create_vertex_array(&self) -> Result<Self::VertexArray, String> {
        Ok(())
    }

    unsafe fn delete_vertex_array(&self, _vertex_array: Self::VertexArray) {}

    unsafe fn bind_vertex_array(&self, _vertex_array: Option<Self::VertexArray>) {}

    unsafe fn clear_color(&self, _red: f32, _green: f32, _blue: f32, _alpha: f32) {}

    unsafe fn supports_f64_precision() -> bool {
        false
    }

    unsafe fn clear_depth_f64(&self, _depth: f64) {}

    unsafe fn clear_depth_f32(&self, _depth: f32) {}

    unsafe fn clear_stencil(&self, _stencil: i32) {}

    unsafe fn clear(&self, _mask: u32) {}

    unsafe fn patch_parameter_i32(&self, _parameter: u32, _value: i32) {}

    unsafe fn pixel_store_i32(&self, _parameter: u32, _value: i32) {}

    unsafe fn pixel_store_bool(&self, _parameter: u32, _value: bool) {}

    unsafe fn bind_frag_data_location(
        &self,
        _program: Self::Program,
        _color_number: u32,
        _name: &str,
    ) {
    }

    unsafe fn buffer_data_size(&self, _target: u32, _size: i32, _usage: u32) {}

    unsafe fn buffer_data_u8_slice(&self, _target: u32, _data: &[u8], _usage: u32) {}

    unsafe fn buffer_sub_data_u8_slice(&self, _target: u32, _offset: i32, _src_data: &[u8]) {}

    unsafe fn get_buffer_sub_data(&self, _target: u32, _offset: i32, _dst_data: &mut [u8]) {}

    unsafe fn buffer_storage(
        &self,
        _target: u32,
        _size: i32,
        _data: Option<&mut [u8]>,
        _flags: u32,
    ) {
    }

    unsafe fn check_framebuffer_status(&self, _target: u32) -> u32 {
        glow::FRAMEBUFFER_COMPLETE
    }

    unsafe fn clear_buffer_i32_slice(&self, _target: u32, _draw_buffer: u32, _values: &mut [i32]) {}

    unsafe fn clear_buffer_u32_slice(&self, _target: u32, _draw_buffer: u32, _values: &mut [u32]) {}

    unsafe fn clear_buffer_f32_slice(&self, _target: u32, _draw_buffer: u32, _values: &mut [f32]) {}

    unsafe fn clear_buffer_depth_stencil(
        &self,
        _target: u32,
        _draw_buffer: u32,
        _depth: f32,
        _stencil: i32,
    ) {
    }

    unsafe fn client_wait_sync(&self, _fence: Self::Fence, _flags: u32, _timeout: i32) -> u32 {
        glow::ALREADY_SIGNALED
    }

    unsafe fn copy_buffer_sub_data(
        &self,
        _src_target: u32,
        _dst_target: u32,
        _src_offset: i32,
        _dst_offset: i32,
        _size: i32,
    ) {
    }

    unsafe fn delete_buffer(&self, _buffer: Self::Buffer) {}

    unsafe fn delete_framebuffer(&self, _framebuffer: Self::Framebuffer) {}

    unsafe fn delete_renderbuffer(&self, _renderbuffer: Self::Renderbuffer) {}

    unsafe fn delete_sampler(&self, _texture: Self::Sampler) {}

    unsafe fn delete_sync(&self, _fence: Self::Fence) {}

    unsafe fn delete_texture(&self, _texture: Self::Texture) {}

    unsafe fn disable(&self, _parameter: u32) {}

    unsafe fn disable_draw_buffer(&self, _parameter: u32, _draw_buffer: u32) {}

    unsafe fn disable_vertex_attrib_array(&self, _index: u32) {}

    unsafe fn dispatch_compute(&self, _groups_x: u32, _groups_y: u32, _groups_z: u32) {}

    unsafe fn dispatch_compute_indirect(&self, _offset: i32) {}

    unsafe fn draw_arrays(&self, _mode: u32, _first: i32, _count: i32) {}

    unsafe fn draw_arrays_instanced(
        &self,
        _mode: u32,
        _first: i32,
        _count: i32,
        _instance_count: i32,
    ) {
    }

    unsafe fn draw_arrays_instanced_base_instance(
        &self,
        _mode: u32,
        _first: i32,
        _count: i32,
        _instance_count: i32,
        _base_instance: u32,
    ) {
    }

    unsafe fn draw_buffer(&self, _buffer: u32) {}

    unsafe fn draw_buffers(&self, _buffers: &[u32]) {}

    unsafe fn draw_elements(&self, _mode: u32, _count: i32, _element_type: u32, _offset: i32) {}

    unsafe fn draw_elements_base_vertex(
        &self,
        _mode: u32,
        _count: i32,
        _element_type: u32,
        _offset: i32,
        _base_vertex: i32,
    ) {
    }

    unsafe fn draw_elements_instanced(
        &self,
        _mode: u32,
        _count: i32,
        _element_type: u32,
        _offset: i32,
        _instance_count: i32,
    ) {
    }

    unsafe fn draw_elements_instanced_base_vertex(
        &self,
        _mode: u32,
        _count: i32,
        _element_type: u32,
        _offset: i32,
        _instance_count: i32,
        _base_vertex: i32,
    ) {
    }

    unsafe fn draw_elements_instanced_base_vertex_base_instance(
        &self,
        _mode: u32,
        _count: i32,
        _element_type: u32,
        _offset: i32,
        _instance_count: i32,
        _base_vertex: i32,
        _base_instance: u32,
    ) {
    }

    unsafe fn enable(&self, _parameter: u32) {}

    unsafe fn is_enabled(&self, _parameter: u32) -> bool {
        true
    }

    unsafe fn enable_draw_buffer(&self, _parameter: u32, _draw_buffer: u32) {}

    unsafe fn enable_vertex_attrib_array(&self, _index: u32) {}

    unsafe fn flush(&self) {}

    unsafe fn framebuffer_renderbuffer(
        &self,
        _target: u32,
        _attachment: u32,
        _renderbuffer_target: u32,
        _renderbuffer: Option<Self::Renderbuffer>,
    ) {
    }

    unsafe fn framebuffer_texture(
        &self,
        _target: u32,
        _attachment: u32,
        _texture: Option<Self::Texture>,
        _level: i32,
    ) {
    }

    unsafe fn framebuffer_texture_2d(
        &self,
        _target: u32,
        _attachment: u32,
        _texture_target: u32,
        _texture: Option<Self::Texture>,
        _level: i32,
    ) {
    }

    unsafe fn framebuffer_texture_3d(
        &self,
        _target: u32,
        _attachment: u32,
        _texture_target: u32,
        _texture: Option<Self::Texture>,
        _level: i32,
        _layer: i32,
    ) {
    }

    unsafe fn framebuffer_texture_layer(
        &self,
        _target: u32,
        _attachment: u32,
        _texture: Option<Self::Texture>,
        _level: i32,
        _layer: i32,
    ) {
    }

    unsafe fn front_face(&self, _value: u32) {}

    unsafe fn get_error(&self) -> u32 {
        glow::NO_ERROR
    }

    unsafe fn get_parameter_i32(&self, _parameter: u32) -> i32 {
        0
    }

    unsafe fn get_parameter_indexed_i32(&self, _parameter: u32, _index: u32) -> i32 {
        0
    }

    unsafe fn get_parameter_indexed_string(&self, _parameter: u32, _index: u32) -> String {
        String::new()
    }

    unsafe fn get_parameter_string(&self, _parameter: u32) -> String {
        String::new()
    }

    unsafe fn get_uniform_location(
        &self,
        _program: Self::Program,
        _name: &str,
    ) -> Option<Self::UniformLocation> {
        None
    }

    unsafe fn get_attrib_location(&self, _program: Self::Program, _name: &str) -> Option<u32> {
        None
    }

    unsafe fn bind_attrib_location(&self, _program: Self::Program, _index: u32, _name: &str) {}

    unsafe fn get_active_attributes(&self, _program: Self::Program) -> u32 {
        0
    }

    unsafe fn get_active_attribute(
        &self,
        _program: Self::Program,
        _index: u32,
    ) -> Option<ActiveAttribute> {
        None
    }

    unsafe fn get_sync_status(&self, _fence: Self::Fence) -> u32 {
        glow::UNSIGNALED
    }

    unsafe fn is_sync(&self, _fence: Self::Fence) -> bool {
        false
    }

    unsafe fn renderbuffer_storage(
        &self,
        _target: u32,
        _internal_format: u32,
        _width: i32,
        _height: i32,
    ) {
    }

    unsafe fn sampler_parameter_f32(&self, _sampler: Self::Sampler, _name: u32, _value: f32) {}

    unsafe fn sampler_parameter_f32_slice(
        &self,
        _sampler: Self::Sampler,
        _name: u32,
        _value: &mut [f32],
    ) {
    }

    unsafe fn sampler_parameter_i32(&self, _sampler: Self::Sampler, _name: u32, _value: i32) {}

    unsafe fn generate_mipmap(&self, _target: u32) {}

    unsafe fn tex_image_2d(
        &self,
        _target: u32,
        _level: i32,
        _internal_format: i32,
        _width: i32,
        _height: i32,
        _border: i32,
        _format: u32,
        _ty: u32,
        _pixels: Option<&[u8]>,
    ) {
    }

    unsafe fn tex_image_3d(
        &self,
        _target: u32,
        _level: i32,
        _internal_format: i32,
        _width: i32,
        _height: i32,
        _depth: i32,
        _border: i32,
        _format: u32,
        _ty: u32,
        _pixels: Option<&[u8]>,
    ) {
    }

    unsafe fn tex_storage_2d(
        &self,
        _target: u32,
        _levels: i32,
        _internal_format: u32,
        _width: i32,
        _height: i32,
    ) {
    }

    unsafe fn tex_storage_3d(
        &self,
        _target: u32,
        _levels: i32,
        _internal_format: u32,
        _width: i32,
        _height: i32,
        _depth: i32,
    ) {
    }

    unsafe fn uniform_1_i32(&self, _location: Option<&Self::UniformLocation>, _x: i32) {}

    unsafe fn uniform_2_i32(&self, _location: Option<&Self::UniformLocation>, _x: i32, _y: i32) {}

    unsafe fn uniform_3_i32(
        &self,
        _location: Option<&Self::UniformLocation>,
        _x: i32,
        _y: i32,
        _z: i32,
    ) {
    }

    unsafe fn uniform_4_i32(
        &self,
        _location: Option<&Self::UniformLocation>,
        _x: i32,
        _y: i32,
        _z: i32,
        _w: i32,
    ) {
    }

    unsafe fn uniform_1_i32_slice(&self, _location: Option<&Self::UniformLocation>, _v: &[i32; 1]) {
    }

    unsafe fn uniform_2_i32_slice(&self, _location: Option<&Self::UniformLocation>, _v: &[i32; 2]) {
    }

    unsafe fn uniform_3_i32_slice(&self, _location: Option<&Self::UniformLocation>, _v: &[i32; 3]) {
    }

    unsafe fn uniform_4_i32_slice(&self, _location: Option<&Self::UniformLocation>, _v: &[i32; 4]) {
    }

    unsafe fn uniform_1_f32(&self, _location: Option<&Self::UniformLocation>, _x: f32) {}

    unsafe fn uniform_2_f32(&self, _location: Option<&Self::UniformLocation>, _x: f32, _y: f32) {}

    unsafe fn uniform_3_f32(
        &self,
        _location: Option<&Self::UniformLocation>,
        _x: f32,
        _y: f32,
        _z: f32,
    ) {
    }

    unsafe fn uniform_4_f32(
        &self,
        _location: Option<&Self::UniformLocation>,
        _x: f32,
        _y: f32,
        _z: f32,
        _w: f32,
    ) {
    }

    unsafe fn uniform_1_f32_slice(&self, _location: Option<&Self::UniformLocation>, _v: &[f32; 1]) {
    }

    unsafe fn uniform_2_f32_slice(&self, _location: Option<&Self::UniformLocation>, _v: &[f32; 2]) {
    }

    unsafe fn uniform_3_f32_slice(&self, _location: Option<&Self::UniformLocation>, _v: &[f32; 3]) {
    }

    unsafe fn uniform_4_f32_slice(&self, _location: Option<&Self::UniformLocation>, _v: &[f32; 4]) {
    }

    unsafe fn uniform_matrix_2_f32_slice(
        &self,
        _location: Option<&Self::UniformLocation>,
        _transpose: bool,
        _v: &[f32; 4],
    ) {
    }

    unsafe fn uniform_matrix_3_f32_slice(
        &self,
        _location: Option<&Self::UniformLocation>,
        _transpose: bool,
        _v: &[f32; 9],
    ) {
    }

    unsafe fn uniform_matrix_4_f32_slice(
        &self,
        _location: Option<&Self::UniformLocation>,
        _transpose: bool,
        _v: &[f32; 16],
    ) {
    }

    unsafe fn unmap_buffer(&self, _target: u32) {}

    unsafe fn cull_face(&self, _value: u32) {}

    unsafe fn color_mask(&self, _red: bool, _green: bool, _blue: bool, _alpha: bool) {}

    unsafe fn color_mask_draw_buffer(
        &self,
        _buffer: u32,
        _red: bool,
        _green: bool,
        _blue: bool,
        _alpha: bool,
    ) {
    }

    unsafe fn depth_mask(&self, _value: bool) {}

    unsafe fn blend_color(&self, _red: f32, _green: f32, _blue: f32, _alpha: f32) {}

    unsafe fn line_width(&self, _width: f32) {}

    unsafe fn map_buffer_range(
        &self,
        _target: u32,
        _offset: i32,
        _length: i32,
        _access: u32,
    ) -> *mut u8 {
        unimplemented!()
    }

    unsafe fn flush_mapped_buffer_range(&self, _target: u32, _offset: i32, _length: i32) {}

    unsafe fn invalidate_buffer_sub_data(&self, _target: u32, _offset: i32, _length: i32) {}

    unsafe fn polygon_offset(&self, _factor: f32, _units: f32) {}

    unsafe fn polygon_mode(&self, _face: u32, _mode: u32) {}

    unsafe fn finish(&self) {}

    unsafe fn bind_texture(&self, _target: u32, _texture: Option<Self::Texture>) {}

    unsafe fn bind_sampler(&self, _unit: u32, _sampler: Option<Self::Sampler>) {}

    unsafe fn active_texture(&self, _unit: u32) {}

    unsafe fn fence_sync(&self, _condition: u32, _flags: u32) -> Result<Self::Fence, String> {
        Ok(())
    }

    unsafe fn tex_parameter_f32(&self, _target: u32, _parameter: u32, _value: f32) {}

    unsafe fn tex_parameter_i32(&self, _target: u32, _parameter: u32, _value: i32) {}

    unsafe fn tex_parameter_f32_slice(&self, _target: u32, _parameter: u32, _values: &[f32]) {}

    unsafe fn tex_parameter_i32_slice(&self, _target: u32, _parameter: u32, _values: &[i32]) {}

    unsafe fn tex_sub_image_2d_u8_slice(
        &self,
        _target: u32,
        _level: i32,
        _x_offset: i32,
        _y_offset: i32,
        _width: i32,
        _height: i32,
        _format: u32,
        _ty: u32,
        _pixels: Option<&[u8]>,
    ) {
    }

    unsafe fn tex_sub_image_2d_pixel_buffer_offset(
        &self,
        _target: u32,
        _level: i32,
        _x_offset: i32,
        _y_offset: i32,
        _width: i32,
        _height: i32,
        _format: u32,
        _ty: u32,
        _pixel_buffer_offset: i32,
    ) {
    }

    unsafe fn tex_sub_image_3d_u8_slice(
        &self,
        _target: u32,
        _level: i32,
        _x_offset: i32,
        _y_offset: i32,
        _z_offset: i32,
        _width: i32,
        _height: i32,
        _depth: i32,
        _format: u32,
        _ty: u32,
        _pixels: Option<&[u8]>,
    ) {
    }

    unsafe fn tex_sub_image_3d_pixel_buffer_offset(
        &self,
        _target: u32,
        _level: i32,
        _x_offset: i32,
        _y_offset: i32,
        _z_offset: i32,
        _width: i32,
        _height: i32,
        _depth: i32,
        _format: u32,
        _ty: u32,
        _pixel_buffer_offset: i32,
    ) {
    }

    unsafe fn depth_func(&self, _func: u32) {}

    unsafe fn depth_range_f32(&self, _near: f32, _far: f32) {}

    unsafe fn depth_range_f64(&self, _near: f64, _far: f64) {}

    unsafe fn depth_range_f64_slice(&self, _first: u32, _count: i32, _values: &[[f64; 2]]) {}

    unsafe fn scissor(&self, _x: i32, _y: i32, _width: i32, _height: i32) {}

    unsafe fn scissor_slice(&self, _first: u32, _count: i32, _scissors: &[[i32; 4]]) {}

    unsafe fn vertex_attrib_divisor(&self, _index: u32, _divisor: u32) {}

    unsafe fn vertex_attrib_pointer_f32(
        &self,
        _index: u32,
        _size: i32,
        _data_type: u32,
        _normalized: bool,
        _stride: i32,
        _offset: i32,
    ) {
    }

    unsafe fn vertex_attrib_pointer_i32(
        &self,
        _index: u32,
        _size: i32,
        _data_type: u32,
        _stride: i32,
        _offset: i32,
    ) {
    }

    unsafe fn vertex_attrib_pointer_f64(
        &self,
        _index: u32,
        _size: i32,
        _data_type: u32,
        _stride: i32,
        _offset: i32,
    ) {
    }

    unsafe fn viewport(&self, _x: i32, _y: i32, _width: i32, _height: i32) {}

    unsafe fn viewport_f32_slice(&self, _first: u32, _count: i32, _values: &[[f32; 4]]) {}

    unsafe fn blend_equation(&self, _mode: u32) {}

    unsafe fn blend_equation_draw_buffer(&self, _draw_buffer: u32, _mode: u32) {}

    unsafe fn blend_equation_separate(&self, _mode_rgb: u32, _mode_alpha: u32) {}

    unsafe fn blend_equation_separate_draw_buffer(
        &self,
        _buffer: u32,
        _mode_rgb: u32,
        _mode_alpha: u32,
    ) {
    }

    unsafe fn blend_func(&self, _src: u32, _dst: u32) {}

    unsafe fn blend_func_draw_buffer(&self, _draw_buffer: u32, _src: u32, _dst: u32) {}

    unsafe fn blend_func_separate(
        &self,
        _src_rgb: u32,
        _dst_rgb: u32,
        _src_alpha: u32,
        _dst_alpha: u32,
    ) {
    }

    unsafe fn blend_func_separate_draw_buffer(
        &self,
        _draw_buffer: u32,
        _src_rgb: u32,
        _dst_rgb: u32,
        _src_alpha: u32,
        _dst_alpha: u32,
    ) {
    }

    unsafe fn stencil_func(&self, _func: u32, _reference: i32, _mask: u32) {}

    unsafe fn stencil_func_separate(&self, _face: u32, _func: u32, _reference: i32, _mask: u32) {}

    unsafe fn stencil_mask(&self, _mask: u32) {}

    unsafe fn stencil_mask_separate(&self, _face: u32, _mask: u32) {}

    unsafe fn stencil_op(&self, _stencil_fail: u32, _depth_fail: u32, _pass: u32) {}

    unsafe fn stencil_op_separate(
        &self,
        _face: u32,
        _stencil_fail: u32,
        _depth_fail: u32,
        _pass: u32,
    ) {
    }

    unsafe fn debug_message_control(
        &self,
        _source: u32,
        _msg_type: u32,
        _severity: u32,
        _ids: &[u32],
        _enabled: bool,
    ) {
    }

    unsafe fn debug_message_insert<S>(
        &self,
        _source: u32,
        _msg_type: u32,
        _id: u32,
        _severity: u32,
        _msg: S,
    ) where
        S: AsRef<str>,
    {
    }

    unsafe fn debug_message_callback<F>(&self, _callback: F)
    where
        F: FnMut(u32, u32, u32, u32, &str),
    {
    }

    unsafe fn get_debug_message_log(&self, _count: u32) -> Vec<DebugMessageLogEntry> {
        Vec::new()
    }

    unsafe fn push_debug_group<S>(&self, _source: u32, _id: u32, _message: S)
    where
        S: AsRef<str>,
    {
    }

    unsafe fn pop_debug_group(&self) {}

    unsafe fn object_label<S>(&self, _identifier: u32, _name: u32, _label: Option<S>)
    where
        S: AsRef<str>,
    {
    }

    unsafe fn get_object_label(&self, _identifier: u32, _name: u32) -> String {
        String::new()
    }

    unsafe fn object_ptr_label<S>(&self, _sync: Self::Fence, _label: Option<S>)
    where
        S: AsRef<str>,
    {
    }

    unsafe fn get_object_ptr_label(&self, _sync: Self::Fence) -> String {
        String::new()
    }

    unsafe fn get_uniform_block_index(&self, _program: Self::Program, _name: &str) -> Option<u32> {
        None
    }

    unsafe fn uniform_block_binding(&self, _program: Self::Program, _index: u32, _binding: u32) {}

    unsafe fn get_shader_storage_block_index(
        &self,
        _program: Self::Program,
        _name: &str,
    ) -> Option<u32> {
        None
    }

    unsafe fn shader_storage_block_binding(
        &self,
        _program: Self::Program,
        _index: u32,
        _binding: u32,
    ) {
    }

    unsafe fn read_buffer(&self, _src: u32) {}

    unsafe fn read_pixels(
        &self,
        _x: i32,
        _y: i32,
        _width: i32,
        _height: i32,
        _format: u32,
        _gltype: u32,
        _data: &mut [u8],
    ) {
    }
}
