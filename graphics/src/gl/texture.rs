use crate::TextureType;

// TODO: I don't think this actually maps to TextureType correctly because of platform differences with framebuffers
pub fn to_gl(v: TextureType) -> u32 {
    match v {
        TextureType::Tex2D => glow::TEXTURE_2D,
        TextureType::Volume => glow::TEXTURE_3D,
        TextureType::Tex2DArray => glow::TEXTURE_2D_ARRAY,
        TextureType::Cube => glow::TEXTURE_CUBE_MAP,
    }
}