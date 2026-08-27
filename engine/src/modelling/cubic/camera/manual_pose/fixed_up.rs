use geometry::{Orientation, Pose};
use graphics::linear_algebra::UnitVector;
use triangle_intersection::Vector;

#[derive(Debug, Clone, Copy)]
pub struct FixedUp {
    up: UnitVector<3>,
    position: Vector<3>,
    forward: UnitVector<3>,
}

impl Default for FixedUp {
    fn default() -> Self {
        Self {
            up: UnitVector::Y,
            position: Vector::ZERO,
            forward: UnitVector::Z,
            // left:
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

    pub fn motion_up(&self) -> UnitVector<3> {
        self.up
    }

    pub fn view_forward(&self) -> UnitVector<3> {
        self.forward
    }

    pub fn view_left(&self) -> UnitVector<3> {
        self.up
            .v()
            .cross(self.forward.v())
            .try_normalize()
            .unwrap_or_default()
    }

    pub fn view_up(&self) -> UnitVector<3> {
        self.pose.orientation().view_up()
    }

    pub fn look_up(&mut self, angle: f32) {
        let fdotu = Vector::dot(self.view_forward().v(), self.up.v());
        if !((angle > 0.0 && fdotu > Self::COS_NO_CONE)
            || (angle < 0.0 && fdotu < -Self::COS_NO_CONE))
        {
            Orientation::look_up(angle);
        }
    }

    pub fn motion_forward(&self) -> UnitVector<3> {
        match self {
            Self::FixedUp {
                up, orientation, ..
            } => up
                .v()
                .cross(orientation.view_forward().v().cross(up.v()))
                .normalize_or_panic(),
            Self::Flexible { orientation, .. } => orientation.view_forward(),
        }
    }
}
