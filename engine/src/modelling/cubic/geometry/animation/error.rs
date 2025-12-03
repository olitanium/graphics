use utils::error_boilerplate;

use crate::EngError;
use crate::entity::AggregateModelIndex;
use crate::modelling::cubic::SkeletonBoneIndex;

#[derive(Debug, Clone)]
pub enum Error {
    ParentDoesNotExist {
        requested: AggregateModelIndex,
        available: usize,
    },
    UnitDoesNotExist {
        requested: AggregateModelIndex,
        available: usize,
    },
    SignaturesDoNotMatch {
        animation: u64,
        skeleton: u64,
    },
    BoneInParentDoesNotExist {
        unit: AggregateModelIndex,
        requested: SkeletonBoneIndex,
        length: usize,
    },
    BoneDoesNotExist {
        requested: SkeletonBoneIndex,
        length: usize,
    },
}

error_boilerplate!(Error);

impl<NEVER> From<Error> for EngError<NEVER> {
    fn from(value: Error) -> Self {
        Self::Animation(value)
    }
}
