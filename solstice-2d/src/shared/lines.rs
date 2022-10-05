use crate::Shader;
use solstice::mesh::MultiMesh;
use solstice::{
    mesh::{MappedVertexMesh, VertexMesh},
    vertex::{AttributeType, Vertex, VertexFormat},
    Context,
};

const SHADER_SRC: &str = include_str!("lines.glsl");

#[repr(C)]
#[derive(bytemuck::Zeroable, bytemuck::Pod, Vertex, Copy, Clone, Debug)]
pub struct Position {
    point: [f32; 3],
}

#[repr(C)]
#[derive(bytemuck::Zeroable, bytemuck::Pod, Copy, Clone, Debug)]
pub struct LineVertex {
    pub position: [f32; 3],
    pub width: f32,
    pub color: [f32; 4],
}

impl Default for LineVertex {
    fn default() -> Self {
        Self {
            position: [0., 0., 0.],
            width: 1.0,
            color: [1., 1., 1., 1.],
        }
    }
}

impl Vertex for LineVertex {
    fn build_bindings() -> &'static [VertexFormat] {
        const OFFSET: usize = std::mem::size_of::<LineVertex>();
        &[
            VertexFormat {
                name: "position1",
                offset: 0,
                atype: AttributeType::F32F32F32,
                normalize: false,
            },
            VertexFormat {
                name: "width1",
                offset: std::mem::size_of::<[f32; 3]>(),
                atype: AttributeType::F32,
                normalize: false,
            },
            VertexFormat {
                name: "color1",
                offset: std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<f32>(),
                atype: AttributeType::F32F32F32F32,
                normalize: false,
            },
            VertexFormat {
                name: "position2",
                offset: OFFSET,
                atype: AttributeType::F32F32F32,
                normalize: false,
            },
            VertexFormat {
                name: "width2",
                offset: OFFSET + std::mem::size_of::<[f32; 3]>(),
                atype: AttributeType::F32,
                normalize: false,
            },
            VertexFormat {
                name: "color2",
                offset: OFFSET + std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<f32>(),
                atype: AttributeType::F32F32F32F32,
                normalize: false,
            },
        ]
    }
}

#[allow(unused)]
const SEGMENT_VERTS: [Position; 6] = [
    Position {
        point: [0.0, -0.5, 0.],
    },
    Position {
        point: [1.0, -0.5, 0.],
    },
    Position {
        point: [1.0, 0.5, 0.],
    },
    Position {
        point: [0.0, -0.5, 0.],
    },
    Position {
        point: [1.0, 0.5, 0.],
    },
    Position {
        point: [0.0, 0.5, 0.],
    },
];

fn round_cap_join_geometry(resolution: usize) -> Vec<Position> {
    let mut instance_round_round = vec![
        Position {
            point: [0., -0.5, 0.],
        },
        Position {
            point: [0., -0.5, 1.],
        },
        Position {
            point: [0., 0.5, 1.],
        },
        Position {
            point: [0., -0.5, 0.],
        },
        Position {
            point: [0., 0.5, 1.],
        },
        Position {
            point: [0., 0.5, 0.],
        },
    ];

    const PI: f32 = std::f32::consts::PI;

    // Add the left cap.
    for step in 0..resolution {
        let theta0 = PI / 2. + ((step + 0) as f32 * PI) / resolution as f32;
        let theta1 = PI / 2. + ((step + 1) as f32 * PI) / resolution as f32;
        instance_round_round.push(Position {
            point: [0., 0., 0.],
        });
        instance_round_round.push(Position {
            point: [0.5 * theta0.cos(), 0.5 * theta0.sin(), 0.],
        });
        instance_round_round.push(Position {
            point: [0.5 * theta1.cos(), 0.5 * theta1.sin(), 0.],
        });
    }
    // Add the right cap.
    for step in 0..resolution {
        let theta0 = (3. * PI) / 2. + ((step + 0) as f32 * PI) / resolution as f32;
        let theta1 = (3. * PI) / 2. + ((step + 1) as f32 * PI) / resolution as f32;
        instance_round_round.push(Position {
            point: [0., 0., 1.],
        });
        instance_round_round.push(Position {
            point: [0.5 * theta0.cos(), 0.5 * theta0.sin(), 1.],
        });
        instance_round_round.push(Position {
            point: [0.5 * theta1.cos(), 0.5 * theta1.sin(), 1.],
        });
    }

    instance_round_round
}

const BUFFER_SIZE: usize = 10000;

pub struct LineWorkspace {
    segment_geometry: VertexMesh<Position>,
    positions: MappedVertexMesh<LineVertex>,
    offset: usize,
    unmapped: bool,

    shader: Shader,
}

impl LineWorkspace {
    pub fn new(ctx: &mut Context) -> Result<Self, super::GraphicsError> {
        let segment_geometry = round_cap_join_geometry(50);
        let segment_geometry = VertexMesh::with_data(ctx, &segment_geometry)?;
        // let segment_geometry = VertexMesh::with_data(ctx, &SEGMENT_VERTS)?;
        let positions = MappedVertexMesh::new(ctx, BUFFER_SIZE)?;

        let shader = Shader::with(SHADER_SRC, ctx)?;

        Ok(Self {
            segment_geometry,
            positions,
            offset: 0,
            unmapped: false,
            shader,
        })
    }

    pub fn can_buffer(&self, verts: &[LineVertex]) -> bool {
        self.offset + verts.len() < BUFFER_SIZE
    }

    pub fn add_points(&mut self, verts: &[LineVertex]) {
        if self.unmapped {
            self.unmapped = false;
            self.offset = 0;
        }
        self.positions.set_vertices(verts, self.offset);
        self.offset += verts.len()
    }

    pub fn shader(&self) -> &Shader {
        &self.shader
    }

    pub fn inner(&self) -> solstice::Geometry<MultiMesh> {
        use solstice::mesh::*;

        let mesh = self.positions.inner();
        let instance_count = (self.offset as u32).saturating_sub(1);

        let draw_range = 0..(self.segment_geometry.len());
        let attached = self.segment_geometry.attach_with_step(mesh, 1);
        solstice::Geometry {
            mesh: attached,
            draw_range,
            draw_mode: solstice::DrawMode::Triangles,
            instance_count,
        }
    }

    pub fn geometry(&mut self, ctx: &mut Context) -> solstice::Geometry<MultiMesh> {
        use solstice::mesh::*;

        let mesh = self.positions.unmap(ctx);
        let instance_count = (self.offset as u32).saturating_sub(1);
        self.unmapped = true;

        let draw_range = 0..(self.segment_geometry.len());
        let attached = self.segment_geometry.attach_with_step(mesh, 1);
        solstice::Geometry {
            mesh: attached,
            draw_range,
            draw_mode: solstice::DrawMode::Triangles,
            instance_count,
        }
    }
}
