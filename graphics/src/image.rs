use super::texture::{Filter, PixelFormat, Texture, TextureType, Wrap};
use super::Context;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Settings {
    mipmaps: bool,
    linear: bool,
    dip_scale: f32,
}

impl Settings {
    pub fn new(mipmaps: bool, linear: bool, dip_scale: f32) -> Self {
        Self {
            mipmaps,
            linear,
            dip_scale,
        }
    }
}

pub struct Image {
    gl: Rc<RefCell<Context>>,
    handle: super::TextureKey,
    texture: Texture,
    texture_type: TextureType,
}

impl Image {
    pub fn new(
        ctx: Rc<RefCell<Context>>,
        texture_type: TextureType,
        format: PixelFormat,
        width: usize,
        height: usize,
        slices: usize,
        settings: &Settings,
    ) -> Self {
        ctx.borrow().new_debug_group("Create Image");
        let handle = ctx.borrow_mut().new_texture(texture_type);
        let mut image = Self {
            gl: ctx,
            texture_type,
            handle,
            texture: Texture::new(format, width, height, Filter::default(), Wrap::default()),
        };
        image.set_filter(image.texture.filter());
        image.set_wrap(image.texture.wrap());
        image
    }

    pub fn handle(&self) -> super::TextureKey {
        self.handle
    }

    pub fn texture_type(&self) -> TextureType {
        self.texture_type
    }

    pub fn texture(&self) -> Texture {
        self.texture
    }

    pub fn set_texture(&mut self, texture: Texture) {
        self.texture = texture;
    }

    pub fn set_data(&mut self, data: Option<&[u8]>) {
        self.gl
            .borrow_mut()
            .set_texture_data(self.handle, self.texture, self.texture_type, data);
    }

    pub fn set_wrap(&mut self, wrap: Wrap) {
        // TODO: there's a bunch of extra checks to do in here.
        self.gl
            .borrow_mut()
            .set_texture_wrap(self.handle, self.texture_type, wrap);
        self.texture.set_wrap(wrap);
    }

    pub fn set_filter(&mut self, filter: Filter) {
        self.gl
            .borrow_mut()
            .set_texture_filter(self.handle, self.texture_type, filter);
        self.texture.set_filter(filter);
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        self.gl.borrow_mut().destroy_texture(self.handle);
    }
}
