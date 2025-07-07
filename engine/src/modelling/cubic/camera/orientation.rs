use graphics::linear_algebra::{UnitVector, Vector};

use crate::modelling::cubic::geometry::{Orientation, Pose, YieldsPose};

#[derive(Debug, Clone, Copy)]
pub enum CameraPose {
    FixedUp {
        position: Vector<3>,
        up: UnitVector<3>,
        orientation: Orientation,
    },
    Flexible {
        position: Vector<3>,
        orientation: Orientation,
    },
}

impl CameraPose {
    const COS_NO_CONE: f32 = 0.99;

    pub fn orientation(&self) -> Orientation {
        match self {
            Self::FixedUp { orientation, .. } => *orientation,
            Self::Flexible { orientation, .. } => *orientation,
        }
    }

    pub fn orientation_mut(&mut self) -> &mut Orientation {
        match self {
            Self::FixedUp { orientation, .. } => orientation,
            Self::Flexible { orientation, .. } => orientation,
        }
    }

    pub fn position(&self) -> Vector<3> {
        match self {
            Self::FixedUp { position, .. } => *position,
            Self::Flexible { position, .. } => *position,
        }
    }

    pub fn position_mut(&mut self) -> &mut Vector<3> {
        match self {
            Self::FixedUp { position, .. } => position,
            Self::Flexible { position, .. } => position,
        }
    }

    pub fn motion_left(&self) -> UnitVector<3> {
        self.orientation().view_left()
    }

    pub fn look_left(&mut self, angle: f32) {
        match self {
            Self::Flexible { orientation, .. } => orientation.look_left(angle),
            Self::FixedUp {
                up, orientation, ..
            } => {
                let rotation_about_up = Orientation::axis_angle(*up, angle);
                *orientation = orientation.combine(rotation_about_up)
            }
        }
    }

    pub fn motion_up(&self) -> UnitVector<3> {
        match self {
            Self::FixedUp { up, .. } => *up,
            Self::Flexible { orientation, .. } => orientation.view_up(),
        }
    }

    pub fn look_up(&mut self, angle: f32) {
        match self {
            Self::Flexible { orientation, .. } => orientation.look_up(angle),
            Self::FixedUp {
                up, orientation, ..
            } => {
                let fdotu = Vector::dot(orientation.view_forward().v(), up.v());
                if !((angle > 0.0 && fdotu > Self::COS_NO_CONE)
                    || (angle < 0.0 && fdotu < -Self::COS_NO_CONE))
                {
                    orientation.look_up(angle);
                }
            }
        }
    }

    pub fn motion_forward(&self) -> UnitVector<3> {
        match self {
            Self::FixedUp {
                up, orientation, ..
            } => up
                .v()
                .cross(orientation.view_forward().v().cross(up.v()))
                .normalize(),
            Self::Flexible { orientation, .. } => orientation.view_forward(),
        }
    }

    pub fn roll_ccw(&mut self, angle: f32) {
        if let Self::Flexible { orientation, .. } = self { orientation.roll_ccw(angle); }
    }

    pub fn new_fixed_up_from_to(position: Vector<3>, to: Vector<3>, up: UnitVector<3>) -> Self {
        let orientation = Orientation::new_from_to(position, to, up);

        Self::FixedUp {
            position,
            up,
            orientation,
        }
    }

    pub fn new_fixed_forward_up(
        position: Vector<3>,
        forward: UnitVector<3>,
        up: UnitVector<3>,
    ) -> Self {
        let orientation = Orientation::new_forward_up(forward, up);

        Self::FixedUp {
            position,
            up,
            orientation,
        }
    }

    pub fn new_flexible_from_to(position: Vector<3>, to: Vector<3>, up: UnitVector<3>) -> Self {
        let orientation = Orientation::new_from_to(position, to, up);

        Self::Flexible {
            position,
            orientation,
        }
    }

    pub fn new_flexible_forward_up(
        position: Vector<3>,
        forward: UnitVector<3>,
        up: UnitVector<3>,
    ) -> Self {
        let orientation = Orientation::new_forward_up(forward, up);

        Self::Flexible {
            position,
            orientation,
        }
    }

    pub fn set_orientation(&mut self, orientation: Orientation) {
        *self.orientation_mut() = orientation;
    }

    pub fn set_position(&mut self, position: Vector<3>) {
        *self.position_mut() = position;
    }

    pub fn move_left(&mut self, distance: f32) {
        let diff = self.motion_left().v().scale(distance);
        *self.position_mut() += diff;
    }

    pub fn move_up(&mut self, distance: f32) {
        let diff = self.motion_up().v().scale(distance);
        *self.position_mut() += diff;
    }

    pub fn move_forward(&mut self, distance: f32) {
        let diff = self.motion_forward().v().scale(distance);
        *self.position_mut() += diff;
    }
}

impl YieldsPose for CameraPose {
    type Hint = ();

    fn get_pose(&self, _: Self::Hint) -> Pose {
        Pose::new_from_orientation_translation(self.orientation(), self.position())
    }
}
