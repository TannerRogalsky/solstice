use crate::PixelFormat;

#[allow(unused)]
pub fn size(format: PixelFormat) -> usize {
    match format {
        PixelFormat::Unknown => 0,
        PixelFormat::LUMINANCE | PixelFormat::Stencil8 | PixelFormat::Alpha => 1,
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
        PixelFormat::LUMINANCE
        | PixelFormat::R16
        | PixelFormat::R16F
        | PixelFormat::R32F
        | PixelFormat::Alpha => 1,
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

pub fn to_gl(
    format: PixelFormat,
    version: &crate::GLVersion,
    is_renderbuffer: bool,
) -> super::TextureFormat {
    use super::TextureFormat as TF;
    let format = match format {
        PixelFormat::Unknown => panic!("Unknown pixel format!"),
        PixelFormat::LUMINANCE => {
            if version.gles {
                TF {
                    internal: glow::LUMINANCE,
                    external: glow::LUMINANCE,
                    ty: glow::UNSIGNED_BYTE,
                    swizzle: None,
                }
            } else {
                TF {
                    internal: glow::R8,
                    external: glow::RED,
                    ty: glow::UNSIGNED_BYTE,
                    swizzle: Some([
                        glow::RED as _,
                        glow::RED as _,
                        glow::RED as _,
                        glow::ONE as _,
                    ]),
                }
            }
        }
        PixelFormat::Alpha => TF {
            internal: glow::ALPHA,
            external: glow::ALPHA,
            ty: glow::UNSIGNED_BYTE,
            swizzle: None,
        },
        PixelFormat::RG8 => (glow::RG8, glow::RG, glow::UNSIGNED_BYTE).into(),
        PixelFormat::RGB8 => (glow::RGB8, glow::RGB, glow::UNSIGNED_BYTE).into(),
        PixelFormat::RGBA8 => (glow::RGBA8, glow::RGBA, glow::UNSIGNED_BYTE).into(),
        PixelFormat::SRGBA8 => (glow::SRGB8, glow::SRGB, glow::UNSIGNED_BYTE).into(),
        PixelFormat::R16 => (glow::R16, glow::RED, glow::UNSIGNED_SHORT).into(),
        PixelFormat::RG16 => (glow::RG16, glow::RG, glow::UNSIGNED_SHORT).into(),
        PixelFormat::RGBA16 => (glow::RGBA16, glow::RGBA, glow::UNSIGNED_SHORT).into(),
        PixelFormat::R16F => (glow::R16F, glow::RED, glow::HALF_FLOAT).into(),
        PixelFormat::RG16F => (glow::RG16F, glow::RG, glow::HALF_FLOAT).into(),
        PixelFormat::RGBA16F => (glow::RGBA16F, glow::RGBA, glow::HALF_FLOAT).into(),
        PixelFormat::R32F => (glow::R32F, glow::RED, glow::FLOAT).into(),
        PixelFormat::RG32F => (glow::RG32F, glow::RG, glow::FLOAT).into(),
        PixelFormat::RGBA32F => (glow::RGBA32F, glow::RGBA, glow::FLOAT).into(),
        PixelFormat::Stencil8 => (glow::STENCIL_INDEX8, glow::STENCIL, glow::UNSIGNED_BYTE).into(),
        PixelFormat::Depth16 => (
            glow::DEPTH_COMPONENT16,
            glow::DEPTH_COMPONENT,
            glow::UNSIGNED_SHORT,
        )
            .into(),
        PixelFormat::Depth24 => (
            glow::DEPTH_COMPONENT24,
            glow::DEPTH_COMPONENT,
            glow::UNSIGNED_INT_24_8,
        )
            .into(),
        PixelFormat::Depth32F => {
            (glow::DEPTH_COMPONENT32F, glow::DEPTH_COMPONENT, glow::FLOAT).into()
        }
        PixelFormat::Depth24Stencil8 => (
            glow::DEPTH24_STENCIL8,
            glow::DEPTH_STENCIL,
            glow::UNSIGNED_INT_24_8,
        )
            .into(),
        PixelFormat::Depth32fStencil8 => (
            glow::DEPTH32F_STENCIL8,
            glow::DEPTH_STENCIL,
            glow::FLOAT_32_UNSIGNED_INT_24_8_REV,
        )
            .into(),
    };

    if version.gles && !is_renderbuffer {
        TF {
            internal: format.external,
            ..format
        }
    } else {
        format
    }
}
