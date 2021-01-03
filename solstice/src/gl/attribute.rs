use crate::vertex::AttributeType;

pub fn from_gl(atype: u32) -> AttributeType {
    match atype {
        glow::FLOAT => AttributeType::F32,
        glow::FLOAT_VEC2 => AttributeType::F32F32,
        glow::FLOAT_VEC3 => AttributeType::F32F32F32,
        glow::FLOAT_VEC4 => AttributeType::F32F32F32F32,
        glow::FLOAT_MAT2 => AttributeType::F32x2x2,
        glow::FLOAT_MAT3 => AttributeType::F32x3x3,
        glow::FLOAT_MAT4 => AttributeType::F32x4x4,
        glow::INT => AttributeType::I32,
        glow::INT_VEC2 => AttributeType::I32I32,
        glow::INT_VEC3 => AttributeType::I32I32I32,
        glow::INT_VEC4 => AttributeType::I32I32I32I32,
        v => panic!("Unknown value returned by OpenGL attribute type: {:#x}", v),
    }
}
