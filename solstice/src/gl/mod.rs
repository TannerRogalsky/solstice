pub mod attribute;
pub mod draw_mode;
pub mod pixel_format;
pub mod texture;
pub mod vertex_winding;
pub mod wrap_mode;

pub struct TextureFormat {
    pub internal: u32,
    pub external: u32,
    pub ty: u32,
    pub swizzle: Option<[i32; 4]>,
}

impl From<(u32, u32, u32)> for TextureFormat {
    fn from((internal, external, ty): (u32, u32, u32)) -> Self {
        Self {
            internal,
            external,
            ty,
            swizzle: None,
        }
    }
}
