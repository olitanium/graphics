use geometry::{Orientation, Pose, Transform};
use graphics::linear_algebra::UnitVector;
use triangle_intersection::Vector;

use crate::modelling::cubic::CameraPose;

// REMEMBER: left = +x
//             up = +y
//        forward = +z

#[derive(Debug, Clone, Copy)]
pub struct FixedUp {
    position: Vector<3>,
    forward: UnitVector<3>,
    up: UnitVector<3>,
}

impl Default for FixedUp {
    fn default() -> Self {
        Self {
            up: UnitVector::Y,
            position: Vector::ZERO,
            forward: UnitVector::Z,
            // left: UnitVector::X,
        }
    }
}

impl FixedUp {
    const COS_NO_CONE: f32 = 0.99;

    pub fn new_from_to(position: Vector<3>, to: Vector<3>, up: UnitVector<3>) -> Option<Self> {
        let forward = (to - position).try_normalize()?;

        (forward != up).then_some(Self {
            up,
            position,
            forward,
        })
    }

    pub fn new_forward(
        position: Vector<3>,
        forward: UnitVector<3>,
        up: UnitVector<3>,
    ) -> Option<Self> {
        Some(Self {
            up,
            forward,
            position,
        })
    }

    pub fn left(&self) -> UnitVector<3> {
        self.up
            .v()
            .cross(self.forward.v())
            .try_normalize()
            .unwrap_or(UnitVector::X)
    }

    pub fn left_nonunit(&self) -> Vector<3> {
        self.up.v().cross(self.forward.v())
    }

    pub fn motion_up(&self) -> UnitVector<3> {
        self.up
    }

    pub fn view_up(&self, left: Option<Vector<3>>) -> UnitVector<3> {
        let left = left.unwrap_or_else(|| self.left_nonunit());
        let up = self
            .forward
            .v()
            .cross(left)
            .try_normalize()
            .unwrap_or(UnitVector::Y);
        up
    }

    pub fn motion_forward(&self, left: Option<Vector<3>>) -> UnitVector<3> {
        let left = left.unwrap_or_else(|| self.left_nonunit());
        let forward = left
            .cross(self.up.v())
            .try_normalize()
            .unwrap_or(UnitVector::Z);
        forward
    }

    pub fn view_forward(&self) -> UnitVector<3> {
        self.forward
    }

    pub fn look_up(&mut self, angle: f32, left: Option<UnitVector<3>>) {
        let fdotu = self.view_forward().v().dot(self.motion_up().v());
        if (angle > 0.0 && fdotu < Self::COS_NO_CONE) || (angle < 0.0 && fdotu > -Self::COS_NO_CONE)
        {
            let left = left.unwrap_or_else(|| self.left());
            let look_up = Orientation::axis_angle(-left, angle);
            self.forward = look_up.act_on_unit(self.forward)
        }
    }

    pub fn look_left(&mut self, angle: f32) {
        let look_left = Orientation::axis_angle(self.up, angle);
        self.forward = look_left.act_on_unit(self.forward)
    }

    pub fn move_left(&mut self, distance: f32, left: Option<UnitVector<3>>) {
        self.move_in(left.unwrap_or_else(|| self.left()).v().scale(distance));
    }

    pub fn move_forward(&mut self, distance: f32, left: Option<Vector<3>>) {
        self.move_in(self.motion_forward(left).v().scale(distance))
    }

    pub fn move_up(&mut self, distance: f32) {
        self.move_in(self.motion_up().v().scale(distance));
    }

    pub fn move_in(&mut self, displacement: Vector<3>) {
        self.position += displacement
    }

    pub fn move_in_relative(&mut self, relative_displacement: Vector<3>) {
        let displacement = self.get_orientation(&()).transform(relative_displacement);
        self.move_in(displacement);
    }
}

impl CameraPose for FixedUp {
    type Hint = ();

    fn get_orientation(&self, _: &Self::Hint) -> Orientation {
        Orientation::new_forward_up(self.forward, self.up).unwrap_or_default()
    }

    fn get_position(&self, _: &Self::Hint) -> Vector<3> {
        self.position
    }

    fn get_pose(&self, _: &Self::Hint) -> Pose {
        Pose::new_from_orientation_translation(self.get_orientation(&()), self.get_position(&()))
    }
}
