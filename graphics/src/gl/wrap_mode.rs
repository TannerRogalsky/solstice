use crate::texture::WrapMode;

pub fn to_gl(v: WrapMode) -> u32 {
    match v {
        WrapMode::Clamp => glow::CLAMP_TO_EDGE,
        WrapMode::ClampZero => glow::CLAMP_TO_BORDER,
        WrapMode::Repeat => glow::REPEAT,
        WrapMode::MirroredRepeat => glow::MIRRORED_REPEAT,
    }
}