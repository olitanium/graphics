mod error;
pub use error::Error;
// mod included;
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
