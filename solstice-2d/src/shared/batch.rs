#[repr(C)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) struct Transform {
    tx: mint::ColumnMatrix4<f32>,
}

unsafe impl bytemuck::Zeroable for Transform {}
unsafe impl bytemuck::Pod for Transform {}

impl solstice::vertex::Vertex for Transform {
    fn build_bindings() -> &'static [solstice::vertex::VertexFormat] {
        &[solstice::vertex::VertexFormat {
            name: "uModel",
            offset: 0,
            atype: solstice::vertex::AttributeType::F32x4x4,
            normalize: false,
        }]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Base<'a, V> {
    Data(crate::Geometry<'a, V>),
    VertexMesh(solstice::Geometry<&'a solstice::mesh::VertexMesh<V>>),
    IndexedMesh(solstice::Geometry<&'a solstice::mesh::IndexedMesh<V, u32>>),
    IndexedMeshU16(solstice::Geometry<&'a solstice::mesh::IndexedMesh<V, u16>>),
    MultiMesh(solstice::Geometry<&'a solstice::mesh::MultiMesh<'a>>),
}

impl<'a, T, V> From<T> for Base<'a, V>
where
    T: Into<crate::Geometry<'a, V>>,
{
    fn from(data: T) -> Self {
        Self::Data(data.into())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Batch<'a, V> {
    base: Base<'a, V>,
    transforms: Vec<Transform>,
}

impl<'a, V> Batch<'a, V> {
    pub fn new<G>(base: G) -> Self
    where
        V: solstice::vertex::Vertex,
        G: Into<Base<'a, V>> + 'a,
    {
        Self {
            base: base.into(),
            transforms: vec![],
        }
    }

    pub fn push<T: Into<mint::ColumnMatrix4<f32>>>(&mut self, tx: T) {
        self.transforms.push(Transform { tx: tx.into() });
    }
}

impl<'a> Batch<'a, crate::Vertex3D> {
    pub(crate) fn unmap(
        &self,
        ctx: &mut crate::Context,
        meshes: &'a mut crate::GeometryBuffers,
    ) -> solstice::Geometry<solstice::mesh::MultiMesh<'a>> {
        use solstice::mesh::MeshAttacher;

        let instances = &mut meshes.instances;
        instances.set_vertices(ctx, &self.transforms, 0);
        match &self.base {
            Base::Data(data) => {
                let (mesh, draw_range) = match &data.indices {
                    None => {
                        meshes.mesh3d_unindexed.set_vertices(&data.vertices, 0);
                        let mesh = meshes.mesh3d_unindexed.unmap(ctx);
                        (mesh.attach_with_step(instances, 1), 0..data.vertices.len())
                    }
                    Some(indices) => {
                        meshes.mesh3d.set_vertices(&data.vertices, 0);
                        meshes.mesh3d.set_indices(indices, 0);
                        let mesh = meshes.mesh3d.unmap(ctx);
                        (mesh.attach_with_step(instances, 1), 0..indices.len())
                    }
                };
                solstice::Geometry {
                    mesh,
                    draw_range,
                    draw_mode: solstice::DrawMode::Triangles,
                    instance_count: self.transforms.len() as _,
                }
            }
            Base::VertexMesh(geometry) => {
                let mesh = geometry.mesh.attach_with_step(instances, 1);
                solstice::Geometry {
                    mesh,
                    draw_range: geometry.draw_range.clone(),
                    draw_mode: geometry.draw_mode,
                    instance_count: self.transforms.len() as _,
                }
            }
            Base::IndexedMesh(geometry) => {
                let mesh = geometry.mesh.attach_with_step(instances, 1);
                solstice::Geometry {
                    mesh,
                    draw_range: geometry.draw_range.clone(),
                    draw_mode: geometry.draw_mode,
                    instance_count: self.transforms.len() as _,
                }
            }
            Base::IndexedMeshU16(geometry) => {
                let mesh = geometry.mesh.attach_with_step(instances, 1);
                solstice::Geometry {
                    mesh,
                    draw_range: geometry.draw_range.clone(),
                    draw_mode: geometry.draw_mode,
                    instance_count: self.transforms.len() as _,
                }
            }
            Base::MultiMesh(geometry) => {
                let mesh = geometry.mesh.attach_with_step(instances, 1);
                solstice::Geometry {
                    mesh,
                    draw_range: geometry.draw_range.clone(),
                    draw_mode: geometry.draw_mode,
                    instance_count: self.transforms.len() as _,
                }
            }
        }
    }
}

impl<'a> Batch<'a, crate::Vertex2D> {
    pub(crate) fn unmap(
        &self,
        ctx: &mut crate::Context,
        meshes: &'a mut crate::GeometryBuffers,
    ) -> solstice::Geometry<solstice::mesh::MultiMesh<'a>> {
        use solstice::mesh::MeshAttacher;

        let instances = &mut meshes.instances;
        instances.set_vertices(ctx, &self.transforms, 0);
        match &self.base {
            Base::Data(data) => {
                let (mesh, draw_range) = match &data.indices {
                    None => {
                        meshes.mesh2d_unindexed.set_vertices(&data.vertices, 0);
                        let mesh = meshes.mesh2d_unindexed.unmap(ctx);
                        (mesh.attach_with_step(instances, 1), 0..data.vertices.len())
                    }
                    Some(indices) => {
                        meshes.mesh2d.set_vertices(&data.vertices, 0);
                        meshes.mesh2d.set_indices(indices, 0);
                        let mesh = meshes.mesh2d.unmap(ctx);
                        (mesh.attach_with_step(instances, 1), 0..indices.len())
                    }
                };
                solstice::Geometry {
                    mesh,
                    draw_range,
                    draw_mode: solstice::DrawMode::Triangles,
                    instance_count: self.transforms.len() as _,
                }
            }
            Base::VertexMesh(geometry) => {
                let mesh = geometry.mesh.attach_with_step(instances, 1);
                solstice::Geometry {
                    mesh,
                    draw_range: geometry.draw_range.clone(),
                    draw_mode: geometry.draw_mode,
                    instance_count: self.transforms.len() as _,
                }
            }
            Base::IndexedMesh(geometry) => {
                let mesh = geometry.mesh.attach_with_step(instances, 1);
                solstice::Geometry {
                    mesh,
                    draw_range: geometry.draw_range.clone(),
                    draw_mode: geometry.draw_mode,
                    instance_count: self.transforms.len() as _,
                }
            }
            Base::IndexedMeshU16(geometry) => {
                let mesh = geometry.mesh.attach_with_step(instances, 1);
                solstice::Geometry {
                    mesh,
                    draw_range: geometry.draw_range.clone(),
                    draw_mode: geometry.draw_mode,
                    instance_count: self.transforms.len() as _,
                }
            }
            Base::MultiMesh(geometry) => {
                let mesh = geometry.mesh.attach_with_step(instances, 1);
                solstice::Geometry {
                    mesh,
                    draw_range: geometry.draw_range.clone(),
                    draw_mode: geometry.draw_mode,
                    instance_count: self.transforms.len() as _,
                }
            }
        }
    }
}
