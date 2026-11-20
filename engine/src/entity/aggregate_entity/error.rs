use utils::error_boilerplate;

#[derive(Debug)]
#[non_exhaustive]
pub enum PushModelError {
    ParentUnitDoesNotExist,
    BoneInParentDoesNotExist,
    InvariantError,
}

error_boilerplate!(PushModelError);
