use graphics::vertex::Vertex;

#[repr(C, packed)]
#[derive(Vertex, Copy, Clone, Debug)]
pub struct Vertex2D {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

impl Default for Vertex2D {
    fn default() -> Self {
        Self {
            position: [0., 0.],
            color: [1., 1., 1., 1.],
            uv: [0.5, 0.5],
        }
    }
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
