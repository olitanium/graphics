mod error;
pub use error::Error;
mod program;
mod shader;

pub use program::{
    ActiveShaderProgram,
    CullFace,
    IsActiveShaderProgram,
    NameNotFoundError,
    ShaderProgram,
    ShaderProgramContext,
    Uniform,
};
