use crate::{buffers, environment, shader_program, texture};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    Texture(texture::Error),
    Shader(shader_program::Error),
    Buffer(buffers::Error),
    Window(environment::Error),
    Other(String),
    Close,
}

utils::error_boilerplate!(Error);
