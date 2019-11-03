use super::texture::{
    Filter, FilterMode, PixelFormat, Texture, TextureInfo, TextureType, TextureUpdate, Wrap,
};
use super::Context;

pub struct Settings {
    mipmaps: bool,
    linear: bool,
    dpi_scale: f32,
}

impl Settings {
    pub fn new(mipmaps: bool, linear: bool, dpi_scale: f32) -> Self {
        Self {
            mipmaps,
            linear,
            dpi_scale,
        }
    }
}
pub struct Image {
    texture_key: super::TextureKey,
    texture_info: TextureInfo,
    texture_type: TextureType,
}

impl Image {
    pub fn new(
        ctx: &mut Context,
        texture_type: TextureType,
        format: PixelFormat,
        width: usize,
        height: usize,
        slices: usize,
        settings: &Settings,
    ) -> Self {
        assert!(
            texture_type.is_supported(),
            "Unsupported Texture Type: {:?}",
            texture_type
        );
        ctx.new_debug_group("Create Image");
        let texture_key = ctx.new_texture(texture_type);
        let filter = if settings.linear {
            Filter::new(
                FilterMode::Linear,
                FilterMode::Linear,
                if settings.mipmaps {
                    FilterMode::Linear
                } else {
                    FilterMode::None
                },
                0.,
            )
        } else {
            Filter::new(
                FilterMode::Nearest,
                FilterMode::Nearest,
                if settings.mipmaps {
                    FilterMode::Nearest
                } else {
                    FilterMode::None
                },
                0.,
            )
        };
        let wrap = Wrap::default();
        ctx.set_texture_filter(texture_key, texture_type, filter);
        ctx.set_texture_wrap(texture_key, texture_type, wrap);
        Self {
            texture_type,
            texture_key,
            texture_info: TextureInfo::new(
                format,
                (width as f32 * settings.dpi_scale + 0.5) as usize,
                (height as f32 * settings.dpi_scale + 0.5) as usize,
                filter,
                wrap,
            ),
        }
    }

    pub fn set_texture_info(&mut self, texture_info: TextureInfo) {
        self.texture_info = texture_info;
    }
}

impl Texture for Image {
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

impl Texture for &Image {
    fn get_texture_key(&self) -> super::TextureKey {
        Image::get_texture_key(self)
    }

    fn get_texture_type(&self) -> TextureType {
        Image::get_texture_type(self)
    }

    fn get_texture_info(&self) -> TextureInfo {
        Image::get_texture_info(self)
    }
}
