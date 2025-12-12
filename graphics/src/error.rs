use crate::{environment, shader_program, texture, types};

pub type ApiResult<T, E> = core::result::Result<T, ApiError<E>>;

#[derive(Debug, Clone)]
pub enum ApiError<E> {
    Graphics(Vec<types::GLError>),
    //Texture(texture::Error),
    //Shader(shader_program::Error),
    //Window(environment::NewError),
    Engine(Box<E>),
    Close,
}

#[macro_export]
macro_rules! from_api_error {
    ($name:path) => {

impl From<ApiError<!>> for $name {
    fn from(value: $crate::error::ApiError<!>) -> Self {
        Self::Api(value)
    }
}
    };

    (<$($lt:lifetime),*> $name: path) => {
impl<$($lt ,)*> From<ApiError<!>> for $name {
    fn from(value: $crate::error::ApiError<!>) -> Self {
        Self::Api(value)
    }
}

    }
}

impl ApiError<!> {
    pub fn to_any<E>(self) -> ApiError<E> {
        match self {
            Self::Graphics(x) => ApiError::Graphics(x),
            //Self::Texture(x) => ApiError::Texture(x),
            //Self::Window(x) => ApiError::Window(x),
            //Self::Shader(x) => ApiError::Shader(x),
            Self::Engine(_) => unreachable!(),
            Self::Close => ApiError::Close,
        }
    }
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
