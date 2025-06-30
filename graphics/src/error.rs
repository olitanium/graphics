use crate::{buffers, shader_program, texture};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Texture(texture::Error),
    Shader(shader_program::Error),
    Buffer(buffers::Error),
}

utils::error_boilerplate!(Error);
