use graphics::vertex::Vertex;
use graphics_macro::Vertex;

#[repr(C, packed)]
#[derive(Vertex, Copy, Clone, Debug)]
pub struct Vertex2D {
    position: [f32; 2],
    color: [f32; 4],
    uv: [f32; 2],
}

impl Vertex2D {
    pub fn new(position: [f32; 2], color: [f32; 4], uv: [f32; 2]) -> Self {
        Self {
            position,
            color,
            uv,
        }
    }
}
