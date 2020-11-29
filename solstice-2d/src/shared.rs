mod color;
mod lines;
mod shader;

pub use color::*;
pub use lines::*;
pub use shader::*;

#[derive(Debug)]
pub enum GraphicsError {
    ShaderError(ShaderError),
    GraphicsError(solstice::GraphicsError),
}

impl std::fmt::Display for GraphicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for GraphicsError {}

impl From<solstice::GraphicsError> for GraphicsError {
    fn from(err: solstice::GraphicsError) -> Self {
        GraphicsError::GraphicsError(err)
    }
}

impl From<ShaderError> for GraphicsError {
    fn from(err: ShaderError) -> Self {
        GraphicsError::ShaderError(err)
    }
}
