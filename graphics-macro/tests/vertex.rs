use graphics::vertex::{HasSemantics, Semantics, VertexAttrib, VertexBufferDesc, VertexNew};
use graphics_macro::{Semantics, Vertex};

#[test]
fn derive_simple_semantics() {
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
    pub enum Semantics {
        #[sem(name = "position", repr = "[f32; 3]", wrapper = "VertexPosition")]
        Position,
        #[sem(name = "normal", repr = "[f32; 3]", wrapper = "VertexNormal")]
        Normal,
        #[sem(name = "color", repr = "[f32; 4]", wrapper = "VertexColor")]
        Color,
    }

    #[derive(Clone, Copy, Debug, Vertex)]
    #[repr(C)]
    #[vertex(sem = "Semantics")]
    struct Vertex {
        pos: VertexPosition,
        nor: VertexNormal,
        col: VertexColor,
    }

    assert_eq!(Semantics::Position.index(), 0);
    assert_eq!(Semantics::Normal.index(), 1);
    assert_eq!(Semantics::Color.index(), 2);
    assert_eq!("position".parse::<Semantics>(), Ok(Semantics::Position));
    assert_eq!("normal".parse::<Semantics>(), Ok(Semantics::Normal));
    assert_eq!("color".parse::<Semantics>(), Ok(Semantics::Color));
    assert_eq!("bidule".parse::<Semantics>(), Err(()));
    assert_eq!(VertexPosition::SEMANTICS, Semantics::Position);
    assert_eq!(VertexNormal::SEMANTICS, Semantics::Normal);
    assert_eq!(VertexColor::SEMANTICS, Semantics::Color);
    assert_eq!(VertexPosition::new([1., 2., 3.]).repr, [1., 2., 3.]);

    let expected_desc = &[
        VertexBufferDesc::new(
            Semantics::Position,
            <[f32; 3] as VertexAttrib>::VERTEX_ATTRIB_DESC,
        ),
        VertexBufferDesc::new(
            Semantics::Normal,
            <[f32; 3] as VertexAttrib>::VERTEX_ATTRIB_DESC,
        ),
        VertexBufferDesc::new(
            Semantics::Color,
            <[f32; 4] as VertexAttrib>::VERTEX_ATTRIB_DESC,
        ),
    ];

    assert_eq!(Vertex::vertex_desc(), expected_desc);
}