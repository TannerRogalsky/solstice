use crate::DrawMode;

pub fn to_gl(v: DrawMode) -> u32 {
    match v {
        DrawMode::Points => glow::POINTS,
        DrawMode::Lines => glow::LINES,
        DrawMode::LineLoop => glow::LINE_LOOP,
        DrawMode::LineStrip => glow::LINE_STRIP,
        DrawMode::Triangles => glow::TRIANGLES,
        DrawMode::TriangleStrip => glow::TRIANGLE_STRIP,
        DrawMode::TriangleFan => glow::TRIANGLE_FAN,
    }
}
