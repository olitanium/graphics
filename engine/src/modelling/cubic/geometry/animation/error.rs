use utils::error_boilerplate;

use crate::EngError;

#[derive(Debug, Clone)]
pub enum Error {
    ParentDoesNotExist {
        requested: usize,
        available: usize,
    },
    AttachementPointDoesNotExist {
        parent: usize,
        available: usize,
        requested: usize,
    },
    SignaturesDoNotMatch {
        animation: u64,
        skeleton: u64,
    },
    BoneDoesNotExist {
        requested: usize,
        length: usize,
    },
}

error_boilerplate!(Error);

impl<NEVER> From<Error> for EngError<NEVER> {
    fn from(value: Error) -> Self {
        Self::Animation(value)
    }
}
