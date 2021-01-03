#[cfg(feature = "derive")]
pub use solstice_derive::Vertex;

use std::fmt::Debug;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AttributeType {
    F32,
    F32F32,
    F32F32F32,
    F32F32F32F32,
    F32x2x2,
    F32x3x3,
    F32x4x4,

    I32,
    I32I32,
    I32I32I32,
    I32I32I32I32,
}

impl AttributeType {
    pub fn get_size_bytes(self) -> usize {
        use std::mem::size_of;
        match self {
            AttributeType::F32 => size_of::<f32>(),
            AttributeType::F32F32 => 2 * size_of::<f32>(),
            AttributeType::F32F32F32 => 3 * size_of::<f32>(),
            AttributeType::F32F32F32F32 => 4 * size_of::<f32>(),
            AttributeType::F32x2x2 => 4 * size_of::<f32>(),
            AttributeType::F32x3x3 => 9 * size_of::<f32>(),
            AttributeType::F32x4x4 => 16 * size_of::<f32>(),
            AttributeType::I32 => size_of::<i32>(),
            AttributeType::I32I32 => 2 * size_of::<i32>(),
            AttributeType::I32I32I32 => 3 * size_of::<i32>(),
            AttributeType::I32I32I32I32 => 4 * size_of::<i32>(),
        }
    }

    pub fn get_num_components(self) -> usize {
        match self {
            AttributeType::F32 | AttributeType::I32 => 1,
            AttributeType::F32F32 | AttributeType::I32I32 => 2,
            AttributeType::F32F32F32 | AttributeType::I32I32I32 => 3,
            AttributeType::F32F32F32F32 | AttributeType::I32I32I32I32 => 4,
            AttributeType::F32x2x2 => 4,
            AttributeType::F32x3x3 => 9,
            AttributeType::F32x4x4 => 16,
        }
    }

    pub fn to_gl(self) -> (u32, i32, i32) {
        match self {
            AttributeType::F32 => (glow::FLOAT, 1, 1),
            AttributeType::F32F32 => (glow::FLOAT, 2, 1),
            AttributeType::F32F32F32 => (glow::FLOAT, 3, 1),
            AttributeType::F32F32F32F32 => (glow::FLOAT, 4, 1),
            AttributeType::F32x2x2 => (glow::FLOAT, 2, 2),
            AttributeType::F32x3x3 => (glow::FLOAT, 3, 3),
            AttributeType::F32x4x4 => (glow::FLOAT, 4, 4),
            AttributeType::I32 => (glow::INT, 1, 1),
            AttributeType::I32I32 => (glow::INT, 2, 1),
            AttributeType::I32I32I32 => (glow::INT, 3, 1),
            AttributeType::I32I32I32I32 => (glow::INT, 4, 1),
        }
    }
}

#[derive(Debug)]
pub struct VertexFormat {
    pub name: &'static str,
    pub offset: usize,
    pub atype: AttributeType,
    pub normalize: bool,
}

/// Trait for structures that represent a vertex.
pub trait Vertex {
    /// Builds the `VertexFormat` representing the layout of this element.
    fn build_bindings() -> &'static [VertexFormat];
}

macro_rules! impl_vertex_attribute {
    ($t:ty, $q:expr) => {
        impl VertexAttributeType for $t {
            const A_TYPE: AttributeType = $q;
        }
    };
}

pub trait VertexAttributeType {
    const A_TYPE: AttributeType;
}

impl_vertex_attribute!(i32, AttributeType::I32);
impl_vertex_attribute!([i32; 2], AttributeType::I32I32);
impl_vertex_attribute!([i32; 3], AttributeType::I32I32I32);
impl_vertex_attribute!([i32; 4], AttributeType::I32I32I32I32);
impl_vertex_attribute!(f32, AttributeType::F32);
impl_vertex_attribute!([f32; 2], AttributeType::F32F32);
impl_vertex_attribute!([f32; 3], AttributeType::F32F32F32);
impl_vertex_attribute!([f32; 4], AttributeType::F32F32F32F32);
impl_vertex_attribute!([[f32; 2]; 2], AttributeType::F32x2x2);
impl_vertex_attribute!([[f32; 3]; 3], AttributeType::F32x3x3);
impl_vertex_attribute!([[f32; 4]; 4], AttributeType::F32x4x4);
