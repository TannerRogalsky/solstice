#[derive(Copy, Clone, Debug)]
pub enum TextureType {
    Tex2D,
    Volume,
    Tex2DArray,
    Cube,
}

impl TextureType {
    pub fn to_gl(&self) -> u32 {
        match self {
            TextureType::Tex2D => glow::TEXTURE_2D,
            TextureType::Volume => glow::TEXTURE_3D,
            TextureType::Tex2DArray => glow::TEXTURE_2D_ARRAY,
            TextureType::Cube => glow::TEXTURE_CUBE_MAP,
        }
    }

    pub fn to_index(&self) -> usize {
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

    pub fn is_supported(&self) -> bool {
        match self {
            TextureType::Tex2D => true,
            TextureType::Volume => false,
            TextureType::Tex2DArray => false,
            TextureType::Cube => false,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum PixelFormat {
    Unknown,

    // "regular" formats
    R8,
    RG8,
    RGB8,
    RGBA8,
    SRGBA8,
    R16,
    RG16,
    RGBA16,
    R16F,
    RG16F,
    RGBA16F,
    R32F,
    RG32F,
    RGBA32F,

    // depth/stencil formats
    Stencil8,
    Depth16,
    Depth24,
    Depth32F,
    Depth24Stencil8,
    Depth32fStencil8,
}

impl PixelFormat {
    pub fn size(&self) -> usize {
        match self {
            PixelFormat::Unknown => 0,
            PixelFormat::R8 | PixelFormat::Stencil8 => 1,
            PixelFormat::RG8 | PixelFormat::R16 | PixelFormat::R16F | PixelFormat::Depth16 => 2,
            PixelFormat::RGB8 => 3,
            PixelFormat::RGBA8
            | PixelFormat::SRGBA8
            | PixelFormat::RG16
            | PixelFormat::RG16F
            | PixelFormat::R32F
            | PixelFormat::Depth24
            | PixelFormat::Depth32F
            | PixelFormat::Depth24Stencil8 => 4,
            PixelFormat::RGBA16
            | PixelFormat::RGBA16F
            | PixelFormat::RG32F
            | PixelFormat::Depth32fStencil8 => 8,
            PixelFormat::RGBA32F => 16,
        }
    }

    pub fn color_components(&self) -> usize {
        match self {
            PixelFormat::R8 | PixelFormat::R16 | PixelFormat::R16F | PixelFormat::R32F => 1,
            PixelFormat::RG8 | PixelFormat::RG16 | PixelFormat::RG16F | PixelFormat::RG32F => 2,
            PixelFormat::RGB8 => 3,
            PixelFormat::RGBA8
            | PixelFormat::SRGBA8
            | PixelFormat::RGBA16
            | PixelFormat::RGBA16F
            | PixelFormat::RGBA32F => 4,
            _ => 0,
        }
    }

    pub fn to_gl(&self) -> (u32, u32, u32) {
        match self {
            PixelFormat::Unknown => panic!("Unknown pixel format!"),
            PixelFormat::R8 => (glow::R8, glow::RED, glow::UNSIGNED_BYTE),
            PixelFormat::RG8 => (glow::RG8, glow::RG, glow::UNSIGNED_BYTE),
            PixelFormat::RGB8 => (glow::RGB8, glow::RGB, glow::UNSIGNED_BYTE),
            PixelFormat::RGBA8 => (glow::RGB8, glow::RGBA, glow::UNSIGNED_BYTE),
            PixelFormat::SRGBA8 => (glow::SRGB8, glow::SRGB, glow::UNSIGNED_BYTE),
            PixelFormat::R16 => (glow::R16, glow::RED, glow::UNSIGNED_SHORT),
            PixelFormat::RG16 => (glow::RG16, glow::RG, glow::UNSIGNED_SHORT),
            PixelFormat::RGBA16 => (glow::RGBA16, glow::RGBA, glow::UNSIGNED_SHORT),
            PixelFormat::R16F => (glow::R16F, glow::RED, glow::HALF_FLOAT),
            PixelFormat::RG16F => (glow::RG16F, glow::RG, glow::HALF_FLOAT),
            PixelFormat::RGBA16F => (glow::RGBA16F, glow::RGBA, glow::HALF_FLOAT),
            PixelFormat::R32F => (glow::R32F, glow::RED, glow::FLOAT),
            PixelFormat::RG32F => (glow::RG32F, glow::RG, glow::FLOAT),
            PixelFormat::RGBA32F => (glow::RGBA32F, glow::RGBA, glow::FLOAT),
            PixelFormat::Stencil8 => (glow::STENCIL_INDEX8, glow::STENCIL, glow::UNSIGNED_BYTE),
            PixelFormat::Depth16 => (
                glow::DEPTH_COMPONENT16,
                glow::DEPTH_COMPONENT,
                glow::UNSIGNED_SHORT,
            ),
            PixelFormat::Depth24 => (
                glow::DEPTH_COMPONENT24,
                glow::DEPTH_COMPONENT,
                glow::UNSIGNED_INT_24_8,
            ),
            PixelFormat::Depth32F => (glow::DEPTH_COMPONENT32F, glow::DEPTH_COMPONENT, glow::FLOAT),
            PixelFormat::Depth24Stencil8 => (
                glow::DEPTH24_STENCIL8,
                glow::DEPTH_STENCIL,
                glow::UNSIGNED_INT_24_8,
            ),
            PixelFormat::Depth32fStencil8 => (
                glow::DEPTH32F_STENCIL8,
                glow::DEPTH_STENCIL,
                glow::FLOAT_32_UNSIGNED_INT_24_8_REV,
            ),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum WrapMode {
    Clamp,
    ClampZero,
    Repeat,
    MirroredRepeat,
}

impl WrapMode {
    pub fn to_gl(&self) -> u32 {
        match self {
            WrapMode::Clamp => glow::CLAMP_TO_EDGE,
            WrapMode::ClampZero => glow::CLAMP_TO_BORDER,
            WrapMode::Repeat => glow::REPEAT,
            WrapMode::MirroredRepeat => glow::MIRRORED_REPEAT,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum FilterMode {
    None,
    Linear,
    Nearest,
}

#[derive(Copy, Clone, Debug)]
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

    pub fn min(&self) -> FilterMode {
        self.min
    }

    pub fn set_min(&mut self, min: FilterMode) {
        self.min = min;
    }

    pub fn mag(&self) -> FilterMode {
        self.mag
    }

    pub fn set_mag(&mut self, mag: FilterMode) {
        self.mag = mag;
    }

    pub fn mipmap(&self) -> FilterMode {
        self.mipmap
    }

    pub fn set_mipmap(&mut self, mipmap: FilterMode) {
        self.mipmap = mipmap;
    }

    pub fn anisotropy(&self) -> f32 {
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

#[derive(Copy, Clone, Debug)]
pub struct Wrap {
    s: WrapMode,
    t: WrapMode,
    r: WrapMode,
}

impl Wrap {
    pub fn new(s: WrapMode, t: WrapMode, r: WrapMode) -> Self {
        Self { s, t, r }
    }

    pub fn s(&self) -> WrapMode {
        self.s
    }

    pub fn t(&self) -> WrapMode {
        self.t
    }

    pub fn r(&self) -> WrapMode {
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

#[derive(Copy, Clone, Debug)]
pub struct TextureInfo {
    format: PixelFormat,
    width: usize,
    height: usize,
    filter: Filter,
    wrap: Wrap,
}

impl Default for TextureInfo {
    fn default() -> Self {
        Self {
            format: PixelFormat::Unknown,
            width: 0,
            height: 0,
            filter: Default::default(),
            wrap: Default::default(),
        }
    }
}

impl TextureInfo {
    pub fn new(
        format: PixelFormat,
        width: usize,
        height: usize,
        filter: Filter,
        wrap: Wrap,
    ) -> Self {
        Self {
            format,
            width,
            height,
            filter,
            wrap,
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

    pub fn format(&self) -> PixelFormat {
        self.format
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
