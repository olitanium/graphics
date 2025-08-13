use crate::{environment, shader_program, texture, types};

pub type ApiResult<T, E> = core::result::Result<T, ApiError<E>>;

#[derive(Debug, Clone)]
pub enum ApiError<E> {
    Graphics(Vec<types::GLError>),
    Texture(texture::Error),
    Shader(shader_program::Error),
    // Buffer(buffers::Error),
    Window(environment::Error),
    Engine(E),
    Close,
}

// Display IS debug
impl<E: core::fmt::Debug> core::fmt::Display for ApiError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}



// Derive Error for Error
impl<E: std::error::Error> std::error::Error for ApiError<E> {}

impl<E> From<!> for ApiError<E> {
    fn from(value: !) -> Self {
        value
    }
}

impl ApiError<!> {
    pub fn temp_convert_from_never<T>(self) -> ApiError<T> {
        match self {
            Self::Graphics(x) => ApiError::Graphics(x), 
            Self::Texture(x) => ApiError::Texture(x), 
            Self::Shader(x) => ApiError::Shader(x), 
            Self::Window(x) => ApiError::Window(x),
            Self::Close => ApiError::Close,
        }
    } 
}