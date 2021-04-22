use solstice::vertex::VertexFormat;
use solstice_derive::Vertex;

#[test]
fn derive_simple_semantics() {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Vertex, bytemuck::Zeroable, bytemuck::Pod)]
    pub struct TestVertex {
        pub position: [f32; 3],
        pub alpha: f32,
        pub uv: [f32; 2],
    }

    let bindings: &[VertexFormat] = <TestVertex as solstice::vertex::Vertex>::build_bindings();
    assert_eq!(bindings.len(), 3);

    let mut iter = bindings.iter();

    {
        let binding = iter.next().unwrap();
        assert_eq!(binding.name, "position");
        assert_eq!(binding.offset, memoffset::offset_of!(TestVertex, position));
        assert_eq!(binding.atype, solstice::vertex::AttributeType::F32F32F32);
        assert_eq!(binding.normalize, false);
    }

    {
        let binding = iter.next().unwrap();
        assert_eq!(binding.name, "alpha");
        assert_eq!(binding.offset, memoffset::offset_of!(TestVertex, alpha));
        assert_eq!(binding.atype, solstice::vertex::AttributeType::F32);
        assert_eq!(binding.normalize, false);
    }

    {
        let binding = iter.next().unwrap();
        assert_eq!(binding.name, "uv");
        assert_eq!(binding.offset, memoffset::offset_of!(TestVertex, uv));
        assert_eq!(binding.atype, solstice::vertex::AttributeType::F32F32);
        assert_eq!(binding.normalize, false);
    }
}
