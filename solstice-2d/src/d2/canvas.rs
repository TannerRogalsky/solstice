use solstice::texture::{TextureInfo, TextureType};
use solstice::{canvas as s, TextureKey};

#[derive(Clone, Debug, PartialEq)]
pub struct Canvas {
    pub inner: solstice::canvas::Canvas,
}

impl Canvas {
    pub fn new(
        ctx: &mut solstice::Context,
        width: f32,
        height: f32,
    ) -> Result<Self, solstice::GraphicsError> {
        Self::with_settings(
            ctx,
            s::Settings {
                width: width as _,
                height: height as _,
                ..s::Settings::default()
            },
        )
    }

    pub fn with_settings(
        ctx: &mut solstice::Context,
        settings: s::Settings,
    ) -> Result<Self, solstice::GraphicsError> {
        let inner = s::Canvas::new(ctx, settings)?;
        Ok(Self { inner })
    }

    pub fn dimensions(&self) -> (f32, f32) {
        let info = solstice::texture::Texture::get_texture_info(&self.inner);
        (info.width() as _, info.height() as _)
    }
}

impl solstice::texture::Texture for Canvas {
    fn get_texture_key(&self) -> TextureKey {
        self.inner.get_texture_key()
    }

    fn get_texture_type(&self) -> TextureType {
        self.inner.get_texture_type()
    }

    fn get_texture_info(&self) -> TextureInfo {
        self.inner.get_texture_info()
    }
}

impl solstice::texture::Texture for &Canvas {
    fn get_texture_key(&self) -> TextureKey {
        self.inner.get_texture_key()
    }

    fn get_texture_type(&self) -> TextureType {
        self.inner.get_texture_type()
    }

    fn get_texture_info(&self) -> TextureInfo {
        self.inner.get_texture_info()
    }
}
