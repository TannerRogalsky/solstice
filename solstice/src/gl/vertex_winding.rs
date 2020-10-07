use crate::VertexWinding;

#[allow(unused)]
pub fn to_gl(v: VertexWinding) -> u32 {
    match v {
        VertexWinding::ClockWise => glow::CW,
        VertexWinding::CounterClockWise => glow::CCW,
    }
}
