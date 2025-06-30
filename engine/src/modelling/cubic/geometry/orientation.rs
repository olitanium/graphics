mod dual_number;
mod flexible;
pub use dual_number::DualNumber;
pub use flexible::Orientation;

mod quaternion;
pub use dual_number::{DualQuaternion, Sclerp, UnitDualQuaternion};
pub use quaternion::{Quaternion, Slerp};
