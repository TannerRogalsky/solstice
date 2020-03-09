#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TextureType {
    Tex2D,
    Volume,
    Tex2DArray,
    Cube,
}

impl TextureType {
    pub fn to_index(self) -> usize {
        match self {
            TextureType::Tex2D => 0,
            TextureType::Volume => 1,
            TextureType::Tex2DArray => 2,
            TextureType::Cube => 3,
        }
    }

    pub fn enumerate() -> &'static [TextureType] {
        &[
            TextureType::Tex2D,
            TextureType::Volume,
            TextureType::Tex2DArray,
            TextureType::Cube,
        ]
    }

    pub fn is_supported(self) -> bool {
        match self {
            TextureType::Tex2D => true,
            TextureType::Volume => false,
            TextureType::Tex2DArray => false,
            TextureType::Cube => false,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum WrapMode {
    Clamp,
    ClampZero,
    Repeat,
    MirroredRepeat,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FilterMode {
    None,
    Linear,
    Nearest,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Filter {
    min: FilterMode,
    mag: FilterMode,
    mipmap: FilterMode,
    anisotropy: f32,
}

impl Filter {
    pub fn new(min: FilterMode, mag: FilterMode, mipmap: FilterMode, anisotropy: f32) -> Self {
        Self {
            min,
            mag,
            mipmap,
            anisotropy,
        }
    }

    pub fn min(self) -> FilterMode {
        self.min
    }

    pub fn set_min(&mut self, min: FilterMode) {
        self.min = min;
    }

    pub fn mag(self) -> FilterMode {
        self.mag
    }

    pub fn set_mag(&mut self, mag: FilterMode) {
        self.mag = mag;
    }

    pub fn mipmap(self) -> FilterMode {
        self.mipmap
    }

    pub fn set_mipmap(&mut self, mipmap: FilterMode) {
        self.mipmap = mipmap;
    }

    pub fn anisotropy(self) -> f32 {
        self.anisotropy
    }

    pub fn set_anisotropy(&mut self, anisotropy: f32) {
        self.anisotropy = anisotropy;
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            min: FilterMode::Linear,
            mag: FilterMode::Linear,
            mipmap: FilterMode::None,
            anisotropy: 0.0,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Wrap {
    s: WrapMode,
    t: WrapMode,
    r: WrapMode,
}

impl Wrap {
    pub fn new(s: WrapMode, t: WrapMode, r: WrapMode) -> Self {
        Self { s, t, r }
    }

    pub fn s(self) -> WrapMode {
        self.s
    }

    pub fn t(self) -> WrapMode {
        self.t
    }

    pub fn r(self) -> WrapMode {
        self.r
    }
}

impl Default for Wrap {
    fn default() -> Self {
        Self {
            s: WrapMode::Clamp,
            t: WrapMode::Clamp,
            r: WrapMode::Clamp,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TextureInfo {
    format: data::PixelFormat,
    width: usize,
    height: usize,
    filter: Filter,
    wrap: Wrap,
    mipmaps: bool,
}

impl Default for TextureInfo {
    fn default() -> Self {
        Self {
            format: data::PixelFormat::Unknown,
            width: 0,
            height: 0,
            filter: Default::default(),
            wrap: Default::default(),
            mipmaps: false,
        }
    }
}

impl TextureInfo {
    pub fn new(
        format: data::PixelFormat,
        width: usize,
        height: usize,
        filter: Filter,
        wrap: Wrap,
        mipmaps: bool,
    ) -> Self {
        Self {
            format,
            width,
            height,
            filter,
            wrap,
            mipmaps,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn set_width(&mut self, width: usize) {
        self.width = width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn set_height(&mut self, height: usize) {
        self.height = height
    }

    pub fn get_format(&self) -> data::PixelFormat {
        self.format
    }

    pub fn set_format(&mut self, format: data::PixelFormat) {
        self.format = format;
    }

    pub fn wrap(&self) -> Wrap {
        self.wrap
    }

    pub fn set_wrap(&mut self, wrap: Wrap) {
        self.wrap = wrap;
    }

    pub fn filter(&self) -> Filter {
        self.filter
    }

    pub fn set_filter(&mut self, filter: Filter) {
        self.filter = filter;
    }

    pub fn mipmaps(&self) -> bool {
        self.mipmaps
    }

    pub fn set_mipmaps(&mut self, mipmaps: bool) {
        self.mipmaps = mipmaps;
    }
}

pub trait Texture {
    fn get_texture_key(&self) -> super::TextureKey;
    fn get_texture_type(&self) -> TextureType;
    fn get_texture_info(&self) -> TextureInfo;
}

pub trait TextureUpdate {
    fn set_texture_sub_data(
        &mut self,
        texture_key: super::TextureKey,
        texture: TextureInfo,
        texture_type: TextureType,
        data: Option<&[u8]>,
        x_offset: u32,
        y_offset: u32,
    );
    fn set_texture_data(
        &mut self,
        texture_key: super::TextureKey,
        texture: TextureInfo,
        texture_type: TextureType,
        data: Option<&[u8]>,
    );
    fn set_texture_wrap(
        &mut self,
        texture_key: super::TextureKey,
        texture_type: TextureType,
        wrap: Wrap,
    );
    fn set_texture_filter(
        &mut self,
        texture_key: super::TextureKey,
        texture_type: TextureType,
        filter: Filter,
    );
}
