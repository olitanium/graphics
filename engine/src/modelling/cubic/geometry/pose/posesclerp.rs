use dual_number::Sclerp;

use crate::modelling::cubic::geometry::{Pose, YieldsPose};
pub struct PoseSclerp {
    sclerp: Sclerp,
}

impl YieldsPose for PoseSclerp {
    type Hint = f32;

    fn get_pose(&self, hint: Self::Hint) -> super::Pose {
        Pose::new_from_dual(self.sclerp.get(hint))
    }
}

impl PoseSclerp {
    pub fn new_from_sclerp(sclerp: Sclerp) -> Self {
        Self { sclerp }
    }

    pub fn new(from: Pose, to: Pose, duration: f32) -> Self {
        Self { sclerp: Sclerp::new(from.into_inner(), to.into_inner(), duration)}
    }
}