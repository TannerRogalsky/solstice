use super::{
    texture::{
        Filter, FilterMode, Texture, TextureInfo, TextureType, TextureUpdate, Wrap, WrapMode,
    },
    Context,
};
use data::PixelFormat;

pub struct Settings {
    pub mipmaps: bool,
    pub dpi_scale: f32,
    pub slices: usize,
    pub filter: FilterMode,
    pub wrap: WrapMode,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            mipmaps: true,
            dpi_scale: 1.,
            slices: 1,
            filter: FilterMode::Linear,
            wrap: WrapMode::Clamp,
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
        width: u32,
        height: u32,
        settings: Settings,
    ) -> Result<Self, super::GraphicsError> {
        assert!(
            texture_type.is_supported(),
            "Unsupported Texture Type: {:?}",
            texture_type
        );
        let texture_key = ctx.new_texture(texture_type)?;
        let filter = Filter::new(
            settings.filter,
            settings.filter,
            if settings.mipmaps {
                settings.filter
            } else {
                FilterMode::None
            },
            0.,
        );
        let wrap = Wrap::new(settings.wrap, settings.wrap, settings.wrap);
        ctx.set_texture_filter(texture_key, texture_type, filter);
        ctx.set_texture_wrap(texture_key, texture_type, wrap);
        Ok(Self {
            texture_type,
            texture_key,
            texture_info: TextureInfo::new(
                format,
                (width as f32 * settings.dpi_scale + 0.5) as usize,
                (height as f32 * settings.dpi_scale + 0.5) as usize,
                filter,
                wrap,
                settings.mipmaps,
            ),
        })
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
