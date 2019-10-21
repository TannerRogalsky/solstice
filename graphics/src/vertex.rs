#[derive(Copy, Clone, Debug)]
pub enum AttributeType {
    F32,
    F32F32,
    F32F32F32,
    F32F32F32F32,
    F32x2x2,
    F32x3x3,
    F32x4x4,
}

impl AttributeType {
    pub fn get_size_bytes(&self) -> usize {
        use std::mem::size_of;
        match *self {
            AttributeType::F32 => 1 * size_of::<f32>(),
            AttributeType::F32F32 => 2 * size_of::<f32>(),
            AttributeType::F32F32F32 => 3 * size_of::<f32>(),
            AttributeType::F32F32F32F32 => 4 * size_of::<f32>(),
            AttributeType::F32x2x2 => 4 * size_of::<f32>(),
            AttributeType::F32x3x3 => 9 * size_of::<f32>(),
            AttributeType::F32x4x4 => 16 * size_of::<f32>(),
        }
    }

    pub fn get_num_components(&self) -> usize {
        match *self {
            AttributeType::F32 => 1,
            AttributeType::F32F32 => 2,
            AttributeType::F32F32F32 => 3,
            AttributeType::F32F32F32F32 => 4,
            AttributeType::F32x2x2 => 4,
            AttributeType::F32x3x3 => 9,
            AttributeType::F32x4x4 => 16,
        }
    }

    pub fn to_gl(&self) -> (u32, i32, i32) {
        match *self {
            AttributeType::F32 => (glow::FLOAT, 1, 1),
            AttributeType::F32F32 => (glow::FLOAT, 2, 1),
            AttributeType::F32F32F32 => (glow::FLOAT, 3, 1),
            AttributeType::F32F32F32F32 => (glow::FLOAT, 4, 1),
            AttributeType::F32x2x2 => (glow::FLOAT, 2, 2),
            AttributeType::F32x3x3 => (glow::FLOAT, 3, 3),
            AttributeType::F32x4x4 => (glow::FLOAT, 4, 4),
        }
    }
}

pub struct VertexFormat {
    pub name: &'static str,
    pub offset: usize,
    pub atype: AttributeType,
    pub normalize: bool,
}

/// Trait for structures that represent a vertex.
///
/// Instead of implementing this trait yourself, it is recommended to use the `implement_vertex!`
/// macro instead.
// TODO: this should be `unsafe`, but that would break the syntax extension
pub trait Vertex: Copy + Sized {
    /// Builds the `VertexFormat` representing the layout of this element.
    fn build_bindings() -> &'static [VertexFormat];
}

/// Trait for types that can be used as vertex attributes.
pub unsafe trait Attribute: Sized {
    /// Get the type of data.
    fn get_type() -> AttributeType;
}
