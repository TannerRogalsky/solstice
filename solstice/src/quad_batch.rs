use super::{
    mesh::{IndexedMesh, MappedIndexedMesh},
    Context,
};

pub struct QuadIndex(usize);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Quad<T> {
    pub vertices: [T; 4],
}

impl<T> Quad<T>
where
    T: Copy,
{
    pub fn map<N, F>(&self, f: F) -> Quad<N>
    where
        F: Fn(T) -> N,
    {
        Quad {
            vertices: [
                f(self.vertices[0]),
                f(self.vertices[1]),
                f(self.vertices[2]),
                f(self.vertices[3]),
            ],
        }
    }
    pub fn zip<N>(&self, other: Quad<N>) -> Quad<(T, N)>
    where
        N: Copy,
    {
        Quad {
            vertices: [
                (self.vertices[0], other.vertices[0]),
                (self.vertices[1], other.vertices[1]),
                (self.vertices[2], other.vertices[2]),
                (self.vertices[3], other.vertices[3]),
            ],
        }
    }
}

impl<A> From<super::viewport::Viewport<A>> for Quad<(A, A)>
where
    A: Copy + std::ops::Add<Output = A>,
{
    fn from(viewport: super::viewport::Viewport<A>) -> Self {
        let ((x, y), (width, height)) = (viewport.position(), viewport.dimensions());
        let top_left = (x, y);
        let bottom_left = (x, y + height);
        let top_right = (x + width, y);
        let bottom_right = (x + width, y + height);
        Quad {
            vertices: [top_left, bottom_left, bottom_right, top_right],
        }
    }
}

pub const INDICES: [u16; 6] = [0, 1, 3, 1, 2, 3];

/// 0---3
/// | / |
/// 1---2
#[derive(Debug)]
pub struct QuadBatch<T> {
    mesh: MappedIndexedMesh<T, u16>,
    count: usize,
    capacity: usize,
}

impl<T> QuadBatch<T>
where
    T: super::vertex::Vertex + Default + Clone,
{
    pub fn new(gl: &mut Context, capacity: usize) -> Result<Self, super::GraphicsError> {
        let vertex_capacity = capacity * 4;
        let index_capacity = capacity * 6;

        let indices = {
            let mut indices: Vec<u16> = Vec::with_capacity(index_capacity);
            for i in 0..capacity {
                let vi = (i * 4) as u16;
                indices.extend(std::array::IntoIter::new(INDICES).map(|i| vi + i));
            }
            indices
        };

        let mut mesh =
            MappedIndexedMesh::with_data(gl, vec![T::default(); vertex_capacity], indices)?;
        mesh.set_draw_range(Some(0..0));

        Ok(Self {
            mesh,
            count: 0,
            capacity,
        })
    }

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn push(&mut self, quad: Quad<T>) -> QuadIndex {
        assert!(
            self.count < self.capacity,
            "Adding too many quads to QuadBatch"
        );
        let index = QuadIndex(self.count);
        self.mesh.set_vertices(&quad.vertices, self.count * 4);
        self.count += 1;
        self.mesh.set_draw_range(Some(0..(self.count * 6)));
        index
    }

    pub fn get_quad(&self, index: QuadIndex) -> Option<Quad<T>>
    where
        T: std::marker::Copy,
    {
        let index = index.0;
        if index >= self.count {
            None
        } else {
            let index = index * 4;
            let mut vertices = [T::default(); 4];
            for (dst, src) in vertices
                .iter_mut()
                .zip(self.mesh.get_vertices()[index..index + 4].iter())
            {
                *dst = *src;
            }
            Some(Quad { vertices })
        }
    }

    pub fn insert(&mut self, index: QuadIndex, quad: Quad<T>) {
        self.mesh.set_vertices(&quad.vertices, index.0 * 4);
    }

    pub fn clear(&mut self) {
        self.count = 0;
        self.mesh.set_draw_range(Some(0..0));
    }

    pub fn unmap(&mut self, ctx: &mut Context) -> super::Geometry<&IndexedMesh<T, u16>> {
        self.mesh.unmap(ctx);
        self.geometry()
    }

    pub fn geometry(&self) -> super::Geometry<&IndexedMesh<T, u16>> {
        let draw_range = self.mesh.draw_range();
        let mesh = self.mesh.inner();
        super::Geometry {
            mesh,
            draw_range,
            draw_mode: super::DrawMode::Triangles,
            instance_count: 1,
        }
    }
}
