mod boilerplate;
use boilerplate::*;
use rscsg::dim3::*;
use solstice::mesh::VertexMesh;
use solstice_2d::Vertex3D;
use std::time::Duration;

struct CSGExample {
    vertices: VertexMesh<Vertex3D>,
}

impl Example for CSGExample {
    fn new(ctx: &mut ExampleContext) -> eyre::Result<Self> {
        let csg = Csg::subtract(
            &Csg::cube(Vector(1., 1., 1.), true),
            &Csg::cylinder(Vector(-1., 0., 0.), Vector(1., 0., 0.), 0.5, 8),
        );
        let polygons = csg.get_triangles();

        let vertices = polygons
            .into_iter()
            .flat_map(|triangle| {
                let Vector(nx, ny, nz) = triangle.normal;
                std::array::IntoIter::new(triangle.positions).map(move |position| {
                    let Vector(x, y, z) = position;
                    Vertex3D {
                        position: [x, y, z],
                        uv: [0., 0.],
                        color: [(nx + 1.) / 2., (ny + 1.) / 2., (nz + 1.) / 2., 1.],
                        normal: [nx, ny, nz],
                    }
                })
            })
            .collect::<Vec<_>>();
        let vertices = VertexMesh::with_data(&mut ctx.ctx, &vertices)?;

        Ok(Self { vertices })
    }

    fn draw(&mut self, ctx: &mut ExampleContext, time: Duration) {
        use solstice_2d::*;

        let mut dl = DrawList::default();
        dl.clear(Color::new(0.3, 0.3, 0.3, 1.));

        let mut transform = Transform3D::translation(0., 0., -2.);
        transform *= Transform3D::rotation(
            Rad(0.),
            Rad(time.as_secs_f32()),
            Rad(time.as_secs_f32().sin() * 3.14),
        );
        let geometry = solstice::Geometry {
            mesh: &self.vertices,
            draw_range: 0..self.vertices.len(),
            draw_mode: solstice::DrawMode::Triangles,
            instance_count: 1,
        };
        dl.draw_with_transform(geometry, transform);

        ctx.gfx.process(&mut ctx.ctx, &dl);
    }
}

fn main() {
    CSGExample::run();
}
