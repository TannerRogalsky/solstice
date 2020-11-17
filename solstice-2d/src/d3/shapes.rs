#[derive(Debug, Copy, Clone)]
pub struct Box {
    pub width: f32,
    pub height: f32,
    pub depth: f32,
    pub width_segments: u32,
    pub height_segments: u32,
    pub depth_segments: u32,
}

impl Box {
    pub fn new(
        width: f32,
        height: f32,
        depth: f32,
        width_segments: u32,
        height_segments: u32,
        depth_segments: u32,
    ) -> Self {
        Self {
            width,
            height,
            depth,
            width_segments,
            height_segments,
            depth_segments,
        }
    }
}

impl Default for Box {
    fn default() -> Self {
        Self {
            width: 1.,
            height: 1.,
            depth: 1.,
            width_segments: 1,
            height_segments: 1,
            depth_segments: 1,
        }
    }
}

impl super::Geometry for Box {
    type Vertices = std::vec::IntoIter<super::Vertex3D>;
    type Indices = std::vec::IntoIter<u32>;

    fn vertices(&self) -> Self::Vertices {
        let mut vertices = Vec::new();

        build_plane(
            &mut vertices,
            2,
            1,
            0,
            -1.,
            -1.,
            self.depth,
            self.height,
            self.width,
            self.depth_segments,
            self.height_segments,
        );
        build_plane(
            &mut vertices,
            2,
            1,
            0,
            1.,
            -1.,
            self.depth,
            self.height,
            -self.width,
            self.depth_segments,
            self.height_segments,
        );
        build_plane(
            &mut vertices,
            0,
            2,
            1,
            1.,
            1.,
            self.width,
            self.depth,
            self.height,
            self.width_segments,
            self.depth_segments,
        );
        build_plane(
            &mut vertices,
            0,
            2,
            1,
            1.,
            -1.,
            self.width,
            self.depth,
            -self.height,
            self.width_segments,
            self.depth_segments,
        );
        build_plane(
            &mut vertices,
            0,
            1,
            2,
            1.,
            -1.,
            self.width,
            self.height,
            self.depth,
            self.width_segments,
            self.height_segments,
        );
        build_plane(
            &mut vertices,
            0,
            1,
            2,
            -1.,
            -1.,
            self.width,
            self.height,
            -self.depth,
            self.width_segments,
            self.height_segments,
        );

        vertices.into_iter()
    }

    fn indices(&self) -> Self::Indices {
        let faces = [
            (self.depth_segments, self.height_segments),
            (self.depth_segments, self.height_segments),
            (self.width_segments, self.depth_segments),
            (self.width_segments, self.depth_segments),
            (self.width_segments, self.height_segments),
            (self.width_segments, self.height_segments),
        ];

        let mut index_start = 0;
        let mut indices = Vec::new();
        for (grid_x, grid_y) in faces.iter().copied() {
            let grid_x1 = grid_x + 1;
            for iy in 0..grid_y {
                for ix in 0..grid_x {
                    let a = ix + grid_x1 * iy;
                    let b = ix + grid_x1 * (iy + 1);
                    let c = (ix + 1) + grid_x1 * (iy + 1);
                    let d = (ix + 1) + grid_x1 * iy;

                    indices.push(index_start + a);
                    indices.push(index_start + b);
                    indices.push(index_start + d);

                    indices.push(index_start + b);
                    indices.push(index_start + c);
                    indices.push(index_start + d);
                }
            }
            index_start += grid_x * grid_y * 4;
        }

        indices.into_iter()
    }
}

fn build_plane(
    vertices: &mut Vec<super::Vertex3D>,
    u: usize,
    v: usize,
    w: usize,
    u_dir: f32,
    v_dir: f32,
    width: f32,
    height: f32,
    depth: f32,
    grid_x: u32,
    grid_y: u32,
) {
    let segment_width = width / grid_x as f32;
    let segment_height = height / grid_y as f32;

    let width_half = width / 2.;
    let height_half = height / 2.;
    let depth_half = depth / 2.;

    let grid_x1 = grid_x + 1;
    let grid_y1 = grid_y + 1;

    for iy in 0..grid_y1 {
        let iy = iy as f32;
        let y = iy * segment_height - height_half;
        for ix in 0..grid_x1 {
            let ix = ix as f32;
            let x = ix * segment_width - width_half;

            let mut position = [0f32; 3];
            position[u] = x * u_dir;
            position[v] = y * v_dir;
            position[w] = depth_half;

            let mut normal = [0f32; 3];
            normal[u] = 0.;
            normal[v] = 0.;
            normal[w] = if depth > 0. { 1. } else { -1. };

            vertices.push(super::Vertex3D {
                position,
                normal,
                uv: [ix / grid_x as f32, iy / grid_y as f32],
                color: [1., 1., 1., 1.],
            })
        }
    }
}
