use geometry::{Orientation, Pose, TransformBy};
use graphics::linear_algebra::UnitVector;
use triangle_intersection::Vector;

use crate::modelling::cubic::CameraPose;

// REMEMBER: left = +x
//             up = +y
//        forward = +z

#[derive(Debug, Clone, Copy, Default)]
pub struct Flexible {
    pose: Pose,
}

impl Flexible {
    const COS_NO_CONE: f32 = 0.99;

    pub fn new_from_to(position: Vector<3>, to: Vector<3>, up: UnitVector<3>) -> Option<Self> {
        // let forward = (to - position).try_normalize()?;
        //
        // (forward != up).then_some(Self {
        // up,
        // position,
        // forward,
        // })
        let pose = Pose::new_from_to_up(position, to, up)?;
        Some(Self { pose })
    }

    pub fn new_forward(
        position: Vector<3>,
        forward: UnitVector<3>,
        up: UnitVector<3>,
    ) -> Option<Self> {
        let pose = Pose::new_forward_up(position, forward, up)?;
        Some(Self { pose })
    }

    pub fn left(&self) -> UnitVector<3> {
        self.pose.transform_x()
    }

    pub fn up(&self) -> UnitVector<3> {
        self.pose.transform_y()
    }

    pub fn forward(&self) -> UnitVector<3> {
        self.pose.transform_z()
    }

    pub fn look_up(&mut self, angle: f32) {
        let look_up = Orientation::axis_angle(-UnitVector::X, angle);
        let transform = Pose::new_from_orientation(look_up);
        self.pose = self.pose.transform_by(&transform);
    }

    pub fn look_left(&mut self, angle: f32) {
        let look_left = Orientation::axis_angle(UnitVector::Y, angle);
        let transform = Pose::new_from_orientation(look_left);
        self.pose = self.pose.transform_by(&transform);
    }

    pub fn move_in(&mut self, displacement: Vector<3>) {
        let pose = Pose::new_from_translation(displacement);
        self.pose = self.pose.transform_by(&pose);
    }

    pub fn move_left(&mut self, distance: f32, left: Option<UnitVector<3>>) {
        let left = left.unwrap_or_else(|| self.left()).v().scale(distance);
        self.move_in(left);
    }

    pub fn move_forward(&mut self, distance: f32, forward: Option<UnitVector<3>>) {
        let forward = forward
            .unwrap_or_else(|| self.forward())
            .v()
            .scale(distance);
        self.move_in(forward);
    }

    pub fn move_up(&mut self, distance: f32, up: Option<UnitVector<3>>) {
        let up = up.unwrap_or_else(|| self.up()).v().scale(distance);
        self.move_in(up);
    }
}

impl CameraPose for Flexible {
    type Hint = ();

    fn get_pose(&self, _: &Self::Hint) -> Pose {
        self.pose
    }
}
