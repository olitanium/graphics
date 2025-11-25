use graphics::ApiError;

use crate::modelling::BoundingBoxError;
use crate::modelling::cubic::geometry::AnimationError;

pub type EngResult<T, U> = core::result::Result<T, EngError<U>>;

#[derive(Debug, Clone)]
pub enum EngError<U> {
    User(U),
    Api(graphics::error::ApiError<Self>),
    Import(crate::modelling::ImportError),
    Animation(AnimationError),
    BoundingBox(BoundingBoxError),
}

// Display IS debug
impl<U: core::fmt::Debug> core::fmt::Display for EngError<U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

// Derive Error for Error
impl<U: core::error::Error> core::error::Error for EngError<U> {}

impl<U> From<!> for EngError<U> {
    fn from(value: !) -> Self {
        value
    }
}

impl<U> From<ApiError<Self>> for EngError<U> {
    fn from(value: ApiError<EngError<U>>) -> Self {
        Self::Api(value)
    }
}

impl<U> From<EngError<U>> for ApiError<EngError<U>> {
    fn from(value: EngError<U>) -> Self {
        match value {
            EngError::Api(x) => x,
            other => Self::Engine(Box::new(other)),
        }
    }
}
