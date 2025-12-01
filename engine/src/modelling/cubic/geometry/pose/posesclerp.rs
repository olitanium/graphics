use dual_number::Sclerp;

use crate::modelling::cubic::geometry::Pose;
pub struct PoseSclerp {
    sclerp: Sclerp,
}

impl PoseSclerp {
    pub fn get_pose(&self, prop: f32) -> super::Pose {
        Pose::new_from_dual(self.sclerp.get(prop))
    }

    pub const fn new_from_sclerp(sclerp: Sclerp) -> Self {
        Self { sclerp }
    }

    pub fn new(from: Pose, to: Pose, duration: f32, shortest_path: bool) -> Self {
        Self {
            sclerp: Sclerp::new(from.into_inner(), to.into_inner(), duration, shortest_path),
        }
    }
}
