use super::{
    texture::{Filter, Texture, TextureInfo, TextureType, TextureUpdate, Wrap},
    Context, PixelFormat,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MipmapMode {
    None,
    Manual, // todo: no functional difference between manual and auto right now
    Auto,
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub width: u32,
    pub height: u32,
    pub layers: usize,
    pub mipmap_mode: MipmapMode,
    pub format: PixelFormat,
    pub texture_type: TextureType,
    pub dpi_scale: f32,
    pub msaa: usize,
    pub readable: Option<bool>,
    pub wrap: Wrap,
    pub filter: Filter,
    pub with_depth: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            layers: 1,
            mipmap_mode: MipmapMode::None,
            format: PixelFormat::RGBA8,
            texture_type: TextureType::Tex2D,
            dpi_scale: 1.0,
            msaa: 0,
            readable: None,
            wrap: Default::default(),
            filter: Default::default(),
            with_depth: false,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Canvas {
    framebuffer_key: super::FramebufferKey,
    texture_key: super::TextureKey,
    renderbuffer_key: Option<super::RenderbufferKey>,
    texture_info: TextureInfo,
    texture_type: TextureType,
}

impl Canvas {
    pub fn new(ctx: &mut Context, settings: Settings) -> Result<Self, super::GraphicsError> {
        let texture = TextureInfo::new(
            settings.format,
            (settings.width as f32 * settings.dpi_scale + 0.5) as u32,
            (settings.height as f32 * settings.dpi_scale + 0.5) as u32,
            settings.filter,
            settings.wrap,
            settings.mipmap_mode != MipmapMode::None,
        );
        let (framebuffer_key, texture_key, renderbuffer_key) = {
            let texture_key = ctx.new_texture(settings.texture_type)?;
            ctx.bind_texture_to_unit(settings.texture_type, texture_key, 0.into());
            ctx.set_texture_wrap(texture_key, settings.texture_type, texture.wrap());
            ctx.set_texture_filter(texture_key, settings.texture_type, texture.filter());
            // set format
            ctx.set_texture_data(texture_key, texture, settings.texture_type, None);

            let target = Target::All;
            let current_framebuffer = ctx.get_active_framebuffer(target);

            let framebuffer_key = {
                let framebuffer_key = ctx.new_framebuffer()?;
                ctx.bind_framebuffer(target, Some(framebuffer_key));

                ctx.framebuffer_texture(
                    target,
                    Attachment::Color,
                    settings.texture_type,
                    texture_key,
                    0,
                );
                ctx.clear_color(0., 0., 0., 0.);
                ctx.clear();

                match ctx.check_framebuffer_status(target) {
                    Status::Complete => (),
                    status => {
                        ctx.destroy_framebuffer(framebuffer_key);
                        panic!("Failed to create framebuffer: {:?}", status);
                    }
                }

                framebuffer_key
            };

            let renderbuffer_key = if settings.with_depth {
                let depth_buffer_key = ctx.new_renderbuffer()?;
                ctx.bind_renderbuffer(Some(depth_buffer_key));
                ctx.renderbuffer_storage(
                    PixelFormat::Depth16,
                    texture.width() as _,
                    texture.height() as _,
                );
                ctx.framebuffer_renderbuffer(Attachment::Depth, Some(depth_buffer_key));
                Some(depth_buffer_key)
            } else {
                None
            };

            ctx.bind_framebuffer(target, current_framebuffer);

            (framebuffer_key, texture_key, renderbuffer_key)
        };
        Ok(Self {
            texture_type: settings.texture_type,
            framebuffer_key,
            renderbuffer_key,
            texture_key,
            texture_info: texture,
        })
    }

    pub fn get_framebuffer_key(&self) -> super::FramebufferKey {
        self.framebuffer_key
    }
}

impl Texture for Canvas {
    fn get_texture_key(&self) -> super::TextureKey {
        self.texture_key
    }

    fn get_texture_type(&self) -> TextureType {
        self.texture_type
    }

    fn get_texture_info(&self) -> TextureInfo {
        self.texture_info
    }
}

impl Texture for &Canvas {
    fn get_texture_key(&self) -> super::TextureKey {
        Canvas::get_texture_key(self)
    }

    fn get_texture_type(&self) -> TextureType {
        Canvas::get_texture_type(self)
    }

    fn get_texture_info(&self) -> TextureInfo {
        Canvas::get_texture_info(self)
    }
}

pub enum Attachment {
    Color,
    Depth,
    Stencil,
}

impl Attachment {
    pub fn to_gl(&self) -> u32 {
        match self {
            Attachment::Color => glow::COLOR_ATTACHMENT0,
            Attachment::Depth => glow::DEPTH_ATTACHMENT,
            Attachment::Stencil => glow::STENCIL_ATTACHMENT,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Target {
    Draw,
    Read,
    All,
}

impl Target {
    pub fn to_gl(self) -> u32 {
        match self {
            Target::Draw => glow::DRAW_FRAMEBUFFER,
            Target::Read => glow::READ_FRAMEBUFFER,
            Target::All => glow::FRAMEBUFFER,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Status {
    Complete,
    IncompleteAttachment,
    MissingAttachment,
    //    IncompleteDimensions, TODO: Add support for this to Glow https://github.com/cginternals/glbinding/issues/216
    Unsupported,
    IncompleteMultisample,
    Unknown,
}
