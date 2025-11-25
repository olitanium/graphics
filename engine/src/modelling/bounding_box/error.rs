use utils::error_boilerplate;

use crate::EngError;
use crate::modelling::cubic::geometry::AnimationError;

#[derive(Debug, Clone)]
pub enum Error {
    BadAnimation {
        side: &'static str,
        error: AnimationError,
    },
}

error_boilerplate!(Error);

impl<NEVER> From<Error> for EngError<NEVER> {
    fn from(value: Error) -> Self {
        EngError::BoundingBox(value)
    }
}
