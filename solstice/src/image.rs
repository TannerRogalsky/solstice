use super::PixelFormat;
use super::{
    buffer::Mapped,
    texture::{
        Filter, FilterMode, Texture, TextureInfo, TextureType, TextureUpdate, Wrap, WrapMode,
    },
    viewport::Viewport,
    Context,
};

#[derive(Copy, Clone, Debug)]
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
                (width as f32 * settings.dpi_scale + 0.5) as u32,
                (height as f32 * settings.dpi_scale + 0.5) as u32,
                filter,
                wrap,
                settings.mipmaps,
            ),
        })
    }

    pub fn with_data(
        ctx: &mut Context,
        texture_type: TextureType,
        format: PixelFormat,
        width: u32,
        height: u32,
        data: &[u8],
        settings: Settings,
    ) -> Result<Self, super::GraphicsError> {
        let this = Image::new(ctx, texture_type, format, width, height, settings)?;
        ctx.set_texture_data(
            this.texture_key,
            this.texture_info,
            this.texture_type,
            Some(data),
        );
        Ok(this)
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

pub type MappedImage = Mapped<Image, ndarray::Ix2>;

impl MappedImage {
    pub fn new(
        ctx: &mut Context,
        texture_type: TextureType,
        format: PixelFormat,
        width: u32,
        height: u32,
        settings: Settings,
    ) -> Result<Self, super::GraphicsError> {
        let inner = Image::new(ctx, texture_type, format, width, height, settings)?;
        let pixel_stride = super::gl::pixel_format::size(inner.texture_info.get_format());
        Ok(Self::with_shape(
            inner,
            [height as usize, width as usize * pixel_stride],
        ))
    }

    pub fn with_data(
        ctx: &mut Context,
        texture_type: TextureType,
        format: PixelFormat,
        width: u32,
        height: u32,
        data: Vec<u8>,
        settings: Settings,
    ) -> Result<Self, super::GraphicsError> {
        let inner = Image::with_data(ctx, texture_type, format, width, height, &data, settings)?;
        let pixel_stride = super::gl::pixel_format::size(inner.texture_info.get_format());
        Ok(Self {
            inner,
            memory_map: ndarray::Array2::from_shape_vec(
                [height as usize, width as usize * pixel_stride],
                data,
            )
            .unwrap(),
            modified_range: None,
        })
    }

    pub fn set_pixels(&mut self, region: Viewport<usize>, data: &[u8]) {
        let pixel_stride = self.pixel_stride();
        let (v_width, v_height) = region.dimensions();
        let (x1, y1) = region.position();
        let (x1, y1) = (x1 * pixel_stride, y1);
        let (x2, y2) = (x1 + v_width * pixel_stride, y1 + v_height);
        assert_eq!(v_width * v_height * pixel_stride, data.len());
        let mut slice = self.memory_map.slice_mut(ndarray::s![y1..y2, x1..x2]);
        let data =
            ndarray::ArrayView2::from_shape([v_height, v_width * pixel_stride], data).unwrap();
        slice.assign(&data);
    }

    pub fn get_pixels(&self) -> &[u8] {
        self.memory_map()
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> &[u8] {
        let pixel_stride = self.pixel_stride();
        let index = y * self.inner.texture_info.width() as usize * pixel_stride + x * pixel_stride;
        &self.memory_map()[index..(index + pixel_stride)]
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, pixel: &[u8]) {
        let pixel_stride = self.pixel_stride();
        assert_eq!(pixel_stride, pixel.len());
        let region = Viewport::new(x, y, 1, 1);
        self.set_pixels(region, pixel)
    }

    pub fn pixel_stride(&self) -> usize {
        super::gl::pixel_format::size(self.inner.texture_info.get_format())
    }

    pub fn unmap(&mut self, ctx: &mut Context) -> &Image {
        // TODO, track modified range and texture sub data
        ctx.set_texture_data(
            self.inner.texture_key,
            self.inner.texture_info,
            self.inner.texture_type,
            Some(self.get_pixels()),
        );
        &self.inner
    }
}
