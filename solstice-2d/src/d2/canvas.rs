use solstice::texture::{TextureInfo, TextureType};
use solstice::{canvas as s, TextureKey};

pub struct Canvas {
    pub inner: solstice::canvas::Canvas,
}

impl Canvas {
    pub fn new(
        ctx: &mut solstice::Context,
        width: f32,
        height: f32,
    ) -> Result<Self, solstice::GraphicsError> {
        let inner = s::Canvas::new(
            ctx,
            s::Settings {
                width: width as _,
                height: height as _,
                ..s::Settings::default()
            },
        )?;
        Ok(Self { inner })
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
