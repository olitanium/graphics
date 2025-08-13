use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Error {
    BindTooHigh { maximum: usize, requested: usize },
    OpeningTexture { path: PathBuf },
    ParsingTextureImage { path: PathBuf },
    NoTextureDataOrDims,
    CubeMapNotAllSidesDefined,
}

utils::error_boilerplate!(Error);

impl<E> From<Error> for crate::error::ApiError<E> {
    fn from(value: Error) -> Self {
        Self::Texture(value)
    }
}
