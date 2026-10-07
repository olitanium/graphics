use geometry::{Orientation, Pose, Transform as _};
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

    pub const fn new_from_to(
        position: Vector<3>,
        to: Vector<3>,
        up: UnitVector<3>,
    ) -> Option<Self> {
        let forward = (to - position).try_normalize()?;

        (forward != up).then_some(Self {
            position,
            forward,
            up,
        })
    }

    pub const fn new_forward(
        position: Vector<3>,
        forward: UnitVector<3>,
        up: UnitVector<3>,
    ) -> Option<Self> {
        Some(Self {
            position,
            forward,
            up,
        })
    }

    pub const fn left(&self) -> UnitVector<3> {
        self.up
            .v()
            .cross(self.forward.v())
            .try_normalize()
            .unwrap_or(UnitVector::X)
    }

    pub const fn left_nonunit(&self) -> Vector<3> {
        self.up.v().cross(self.forward.v())
    }

    pub const fn motion_up(&self) -> UnitVector<3> {
        self.up
    }

    pub const fn view_up(&self, left: Option<Vector<3>>) -> UnitVector<3> {
        let left = left.unwrap_or_else(const || self.left_nonunit());
        self.forward
            .v()
            .cross(left)
            .try_normalize()
            .unwrap_or(UnitVector::Y)
    }

    pub fn motion_forward(&self, left: Option<Vector<3>>) -> UnitVector<3> {
        let left = left.unwrap_or_else(|| self.left_nonunit());
        left.cross(self.up.v())
            .try_normalize()
            .unwrap_or(UnitVector::Z)
    }

    pub const fn view_forward(&self) -> UnitVector<3> {
        self.forward
    }

    pub fn look_up(&mut self, angle: f32, left: Option<UnitVector<3>>) {
        let fdotu = self.view_forward().v().dot(self.motion_up().v());
        if (angle > 0.0 && fdotu < Self::COS_NO_CONE) || (angle < 0.0 && fdotu > -Self::COS_NO_CONE)
        {
            let left = left.unwrap_or_else(|| self.left());
            let look_up = Orientation::axis_angle(-left, angle);
            self.forward = look_up.act_on_unit(self.forward);
        }
    }

    pub fn look_left(&mut self, angle: f32) {
        let look_left = Orientation::axis_angle(self.up, angle);
        self.forward = look_left.act_on_unit(self.forward);
    }

    pub fn move_left(&mut self, distance: f32, left: Option<UnitVector<3>>) {
        self.move_in(left.unwrap_or_else(|| self.left()).v().scale(distance));
    }

    pub fn move_forward(&mut self, distance: f32, left: Option<Vector<3>>) {
        self.move_in(self.motion_forward(left).v().scale(distance));
    }

    pub const fn move_up(&mut self, distance: f32)
    where
        [f32; 3]: [const] Clone,
    {
        self.move_in(self.motion_up().v().scale(distance));
    }

    pub const fn move_in(&mut self, displacement: Vector<3>)
    where
        [f32; 3]: [const] Clone,
    {
        self.position += displacement;
    }

    pub fn move_in_relative(&mut self, relative_displacement: Vector<3>) {
        let displacement = self.get_orientation(&()).transform(relative_displacement);
        self.move_in(displacement);
    }
}

impl CameraPose for FixedUp {
    type Hint = ();

    fn get_orientation(&self, (): &Self::Hint) -> Orientation {
        Orientation::new_forward_up(self.forward, self.up).unwrap_or_default()
    }

    fn get_position(&self, (): &Self::Hint) -> Vector<3> {
        self.position
    }

    fn get_pose(&self, (): &Self::Hint) -> Pose {
        Pose::new_from_orientation_translation(self.get_orientation(&()), self.get_position(&()))
    }
}
