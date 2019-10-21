use super::texture::{Filter, PixelFormat, Texture, TextureType, Wrap};
use super::Context;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Canvas {
    gl: Rc<RefCell<Context>>,
    texture_key: super::TextureKey,
    framebuffer_key: super::FramebufferKey,
    texture: Texture,
    texture_type: TextureType,
}

impl Canvas {
    pub fn new(
        ctx: Rc<RefCell<Context>>,
        texture_type: TextureType,
        format: PixelFormat,
        width: usize,
        height: usize,
    ) -> Self {
        let texture = Texture::new(format, width, height, Filter::default(), Wrap::default());
        let (framebuffer_key, texture_key) = {
            let mut ctx = ctx.borrow_mut();
            let texture_key = ctx.new_texture(texture_type);
            ctx.bind_texture_to_unit(texture_type, texture_key, 0);
            ctx.set_texture_data(texture_key, texture, texture_type, None);

            let framebuffer_key = ctx.new_framebuffer();
            ctx.bind_framebuffer(glow::FRAMEBUFFER, Some(framebuffer_key));

            ctx.framebuffer_texture(
                Target::All,
                Attachment::Color,
                texture_type,
                texture_key,
                0,
            );
            ctx.clear_color(0., 0., 0., 0.);
            ctx.clear();

            ctx.bind_framebuffer(glow::FRAMEBUFFER, None);
            (framebuffer_key, texture_key)
        };
        Self {
            gl: ctx,
            texture_type,
            framebuffer_key,
            texture_key,
            texture: Texture::new(format, width, height, Filter::default(), Wrap::default()),
        }
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