use dual_number::Sclerp;

use crate::Pose;

#[derive(Debug)]
pub struct PoseSclerp {
    sclerp: Sclerp,
}

impl PoseSclerp {
    #[must_use]
    pub fn get_pose(&self, prop: f32) -> super::Pose {
        Pose::new_from_dual(self.sclerp.get(prop))
    }

    #[must_use]
    pub const fn new_from_sclerp(sclerp: Sclerp) -> Self {
        Self { sclerp }
    }

    #[must_use]
    pub fn new(from: Pose, to: Pose, duration: f32, shortest_path: bool) -> Self {
        Self {
            sclerp: Sclerp::new(from.into_inner(), to.into_inner(), duration, shortest_path),
        }
    }
}
