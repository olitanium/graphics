pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum Error {
    Graphics(graphics::error::Error),
    Import(crate::modelling::ImportError),
}

utils::error_boilerplate!(Error);
