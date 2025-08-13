use graphics::ApiError;

//pub(crate) type InnerEngResult<T> = EngResult<T, !>;
pub type EngResult<T, U> = core::result::Result<T, EngError<U>>;

pub type EngApiResult<T, U> = graphics::ApiResult<T, EngError<U>>;

#[derive(Debug, Clone)]
pub enum EngError<U> {
    User(U),
    Api(Box<graphics::error::ApiError<Self>>),
    Import(crate::modelling::ImportError),
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
    Self::Api(Box::new(value))
}
}

impl<U> From<EngError<U>> for ApiError<EngError<U>> {
fn from(value: EngError<U>) -> Self {
    match value {
        EngError::Api(x) => *x,
        other => Self::Engine(other)
    }
}
}
