use super::{
    mesh::{Index, Mesh},
    Context,
};

pub struct QuadIndex(usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Quad<T> {
    pub vertices: [T; 4],
}

pub struct QuadBatch<T> {
    mesh: Mesh<T>,
    count: usize,
    capacity: usize,
}

impl<T> QuadBatch<T>
where
    T: super::vertex::Vertex,
{
    pub fn new(gl: &mut Context, capacity: usize) -> Self {
        let vertex_capacity = capacity * 4;
        let index_capacity = capacity * 6;

        let mut mesh = Mesh::new(gl, index_capacity);
        mesh.set_draw_range(Some(0..0));

        {
            // 0---2
            // | / |
            // 1---3
            let mut indices: Vec<Index> = Vec::with_capacity(index_capacity);
            for i in 0..capacity {
                let vi = (i * 4) as Index;
                indices.push(vi);
                indices.push(vi + 1);
                indices.push(vi + 2);
                indices.push(vi + 2);
                indices.push(vi + 1);
                indices.push(vi + 3);
            }
            mesh.set_indices(indices.as_slice(), 0);
        }

        Self {
            mesh,
            count: 0,
            capacity,
        }
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn push(&mut self, quad: Quad<T>) -> QuadIndex {
        let index = QuadIndex(self.count);
        self.mesh.set_vertices(&quad.vertices, self.count * 4);
        self.count += 1;
        self.mesh.set_draw_range(Some(0..(self.count * 6)));
        index
    }

    pub fn insert(&mut self, index: QuadIndex, quad: Quad<T>) {
        self.mesh.set_vertices(&quad.vertices, index.0 * 4);
    }

    pub fn clear(&mut self) {
        self.count = 0;
        self.mesh.set_draw_range(Some(0..0));
    }

    pub fn draw(&mut self, gl: &mut Context) {
        self.mesh.draw(gl)
    }
}
