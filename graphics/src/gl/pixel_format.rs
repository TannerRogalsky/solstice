use crate::PixelFormat;

#[allow(unused)]
pub fn size(format: PixelFormat) -> usize {
    match format {
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

#[allow(unused)]
pub fn color_components(format: PixelFormat) -> usize {
    match format {
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

pub fn to_gl(format: PixelFormat, version: &crate::GLVersion) -> (u32, u32, u32) {
    let format = match format {
        PixelFormat::Unknown => panic!("Unknown pixel format!"),
        PixelFormat::R8 => {
            if version.gles {
                (glow::LUMINANCE, glow::LUMINANCE, glow::UNSIGNED_BYTE)
            } else {
                (glow::R8, glow::RED, glow::UNSIGNED_BYTE)
            }
        }
        PixelFormat::RG8 => (glow::RG8, glow::RG, glow::UNSIGNED_BYTE),
        PixelFormat::RGB8 => (glow::RGB8, glow::RGB, glow::UNSIGNED_BYTE),
        PixelFormat::RGBA8 => (glow::RGBA8, glow::RGBA, glow::UNSIGNED_BYTE),
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
    };

    if version.gles {
        (format.1, format.1, format.2)
    } else {
        format
    }
}
