use super::texture::{Filter, PixelFormat, Texture, TextureType, Wrap};
use super::Context;
use std::cell::RefCell;
use std::rc::Rc;

pub enum MipmapMode {
    None,
    Manual,
    Auto,
}

// TODO: builder pattern?
pub struct Settings {
    pub width: usize,
    pub height: usize,
    pub layers: usize,
    pub mipmap_mode: MipmapMode,
    pub format: PixelFormat,
    pub texture_type: TextureType,
    pub dpi_scale: f32,
    pub msaa: usize,
    pub readable: Option<bool>,
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
        }
    }
}

pub struct Canvas {
    gl: Rc<RefCell<Context>>,
    texture_key: super::TextureKey,
    framebuffer_key: super::FramebufferKey,
    texture: Texture,
    texture_type: TextureType,
}

impl Canvas {
    pub fn new(ctx: Rc<RefCell<Context>>, settings: Settings) -> Self {
        let texture = Texture::new(
            settings.format,
            settings.width,
            settings.height,
            Filter::default(),
            Wrap::default(),
        );
        let (framebuffer_key, texture_key) = {
            let mut ctx = ctx.borrow_mut();
            let texture_key = ctx.new_texture(settings.texture_type);
            ctx.bind_texture_to_unit(settings.texture_type, texture_key, 0);
            ctx.set_texture_wrap(texture_key, settings.texture_type, texture.wrap());
            ctx.set_texture_filter(texture_key, settings.texture_type, texture.filter());
            // set format
            ctx.set_texture_data(texture_key, texture, settings.texture_type, None);

            let framebuffer_key = {
                let target = Target::All;
                let current_framebuffer = ctx.get_active_framebuffer(target);
                let framebuffer_key = ctx.new_framebuffer();
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

                ctx.bind_framebuffer(target, current_framebuffer);
                framebuffer_key
            };
            (framebuffer_key, texture_key)
        };
        Self {
            gl: ctx,
            texture_type: settings.texture_type,
            framebuffer_key,
            texture_key,
            texture,
        }
    }

    pub fn bind(&mut self) {
        self.gl
            .borrow_mut()
            .bind_framebuffer(Target::All, Some(self.framebuffer_key));
    }

    pub fn texture_key(&self) -> super::TextureKey {
        self.texture_key
    }

    pub fn texture_type(&self) -> TextureType {
        self.texture_type
    }

    pub fn texture(&self) -> Texture {
        self.texture
    }
}

impl super::texture::BindableTexture for &Canvas {
    fn get_texture_key(&self) -> super::TextureKey {
        self.texture_key
    }

    fn get_texture_type(&self) -> TextureType {
        self.texture_type
    }

    fn get_texture(&self) -> Texture {
        self.texture
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
    pub fn to_gl(&self) -> u32 {
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
