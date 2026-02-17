use utils::error_boilerplate;

#[derive(Debug)]
#[non_exhaustive]
pub enum PushModelError {
    ParentUnitDoesNotExist,
    BoneInParentDoesNotExist,
}

error_boilerplate!(PushModelError);
