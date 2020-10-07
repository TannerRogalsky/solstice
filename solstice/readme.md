# Centralized State Renderer

The current renderer is nice insofar as the type system and some debug assertion help you avoid mistakes but it still represents a large(ish?) API surface with a lot of internal state and functions that expect to be called in specific orders so that things render properly. This isn't C: we can do better.

Our goal is both to create an API that's hard to screw up and easy to debug without sacrificing flexibility or performance. A secondary but also important thing to remember is that centralizing OpenGL state updates to a single function we set ourselves up for much faster data transfer to JS from WASM because it only has to happen once.

## Methodology

Rust's "splat" object constructing helps ease the difficulty of having to provide all relevant state for each draw call. Optional settings (target framebuffer, blend settings, etc) defaulting to None disable that feature and so don't require those settings further simplifying the creation of this state. This style of object construction also allows additional properties to be added later without breaking users (they will use the default automatically).

```rust
// default
let pipeline_settings = PipelineState::default();

// construct from default
let pipeline_settings = PipelineState {
    viewport: viewport::Viewport::new(0, 0, 720, 480),
    ..PipelineState::default()
};

// construct from existing
let pipeline_settings = PipelineState {
    depth_state: Some(DepthState {
        function: DepthFunction::Never,
        ..DepthState::default()
    }),
    ..pipeline_settings
};
```

Having too many constraints on a single structure can be difficult for users, though, as it can be non-obvious which constraint is being violated and how. So it would seem beneficial to continue using the constructs that encapsulate geometry and shaders: Mesh and Shader, respectively. These allow the creation and modification of encapsulated state in a more intuitive.

Additional resources abstractions are Images (readable textures), Canvases (writable and readable textures) which both implement a Texture interface which help with binding to the shader.

## Handwaved API

```rust
pub trait ResourceCreator {
    fn new_buffer(&mut self, size_or_data: Either<usize, &[u8]>, buffer_type: buffer::BufferType, usage: buffer::Usage) -> BufferKey;
    fn destroy_buffer(&mut self, buffer: BufferKey);

    // textures, shaders, framebuffers, etc
}

pub trait Renderer {
    fn clear(&mut self, settings: ClearSettings);
    fn draw<S: Shader, M: Mesh>(&mut self, shader: &S, mesh: &M, settings: &PipelineSettings);
}

pub trait Shader {
    fn attributes(&self) -> &Vec<Attribute>;
    fn uniforms(&self) -> &Vec<Uniform>;
}

pub trait Mesh {
    fn attributes(&self) -> AttachedAttributes;
}
```

```rust
use Mesh as MeshTrait;

pub struct Mesh<V> {
    vbo: Buffer,
    draw_range: Option<std::ops::Range<usize>>,
    draw_mode: super::DrawMode,
    type_marker: std::marker::PhantomData<V>,
}

impl<V> MeshTrait for Mesh<V>
where
V: Vertex
{
    fn attributes(&self) -> AttachedAttributes {
        AttachedAttributes {
            buffer: &self.vbo,
            formats: V::build_bindings(),
            step: 0,
            stride: std::mem::size_of::<V>(),
        }
    }
}
```

```rust
use Shader as ShaderTrait;

pub struct Shader {
    inner: ShaderKey,
    attributes: Vec<Attribute>,
    uniforms: Vec<Uniform>,
}

impl ShaderTrait for Shader {
    fn attributes(&self) -> &Vec<Attribute> {
        &self.attributes
    }
    fn uniforms(&self) -> &Vec<Uniform> {
        &self.uniforms
    }
}
```