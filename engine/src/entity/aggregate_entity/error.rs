use utils::error_boilerplate;

#[derive(Debug)]
pub enum PushModelError {
    ParentUnitDoesNotExist,
    BoneInParentDoesNotExist,
}

error_boilerplate!(PushModelError);
