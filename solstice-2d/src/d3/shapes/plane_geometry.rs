use crate::d3::Vertex3D;

#[derive(Debug, Copy, Clone)]
pub struct Plane {
    pub width: f32,
    pub height: f32,
    pub width_segments: u32,
    pub height_segments: u32,
}

impl Plane {
    pub fn new(width: f32, height: f32, width_segments: u32, height_segments: u32) -> Self {
        Self {
            width,
            height,
            width_segments,
            height_segments,
        }
    }

    fn vertices(&self) -> Vec<Vertex3D> {
        let Self {
            width,
            height,
            width_segments,
            height_segments,
        } = *self;

        let segment_width = width / width_segments as f32;
        let segment_height = height / height_segments as f32;

        let width_half = width / 2.;
        let height_half = height / 2.;

        let grid_x1 = self.width_segments + 1;
        let grid_y1 = self.height_segments + 1;

        (0..grid_y1)
            .flat_map(move |iy| {
                let iy = iy as f32;
                let y = iy * segment_height - height_half;
                (0..grid_x1).map(move |ix| {
                    let ix = ix as f32;
                    let x = ix * segment_width - width_half;

                    let position = [x, y, 0.];
                    let normal = [0., 0., 1.];

                    Vertex3D {
                        position,
                        normal,
                        uv: [
                            ix / self.width_segments as f32,
                            iy / self.height_segments as f32,
                        ],
                        color: [1., 1., 1., 1.],
                    }
                })
            })
            .collect()
    }

    fn indices(&self) -> Vec<u32> {
        let grid_x1 = self.width_segments + 1;
        (0..self.height_segments)
            .flat_map(|iy| {
                (0..self.width_segments).flat_map(move |ix| {
                    let a = ix + grid_x1 * iy;
                    let b = ix + grid_x1 * (iy + 1);
                    let c = (ix + 1) + grid_x1 * (iy + 1);
                    let d = (ix + 1) + grid_x1 * iy;

                    std::array::IntoIter::new([a, b, d, b, c, d])
                })
            })
            .collect()
    }
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            width: 1.,
            height: 1.,
            width_segments: 1,
            height_segments: 1,
        }
    }
}

impl From<&Plane> for crate::Geometry<'_, Vertex3D> {
    fn from(b: &Plane) -> Self {
        Self::new(b.vertices(), Some(b.indices()))
    }
}

impl From<Plane> for crate::Geometry<'_, Vertex3D> {
    fn from(b: Plane) -> Self {
        Self::new(b.vertices(), Some(b.indices()))
    }
}
