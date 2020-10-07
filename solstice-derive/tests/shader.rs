use solstice::shader::*;
use solstice::ShaderKey;
use solstice_derive::Shader;

#[derive(Shader)]
struct TestShader {
    _inner: ShaderKey,
    #[uniform]
    projection: ShaderProjection,
    #[uniform]
    texture0: ShaderTex0,
}

struct ShaderProjection {
    location: Option<UniformLocation>,
}

impl UniformTrait for ShaderProjection {
    type Value = [f32; 16];
    const NAME: &'static str = "projection";

    fn get_location(&self) -> Option<&UniformLocation> {
        self.location.as_ref()
    }
}

struct ShaderTex0 {
    location: Option<UniformLocation>,
}

impl UniformTrait for ShaderTex0 {
    type Value = Box<dyn solstice::texture::Texture>;
    const NAME: &'static str = "tex0";

    fn get_location(&self) -> Option<&UniformLocation> {
        self.location.as_ref()
    }
}

#[test]
fn example() {
    let _t = TestShader {
        _inner: Default::default(),
        projection: ShaderProjection { location: None },
        texture0: ShaderTex0 { location: None },
    };
}
