use super::quaternion::Quaternion;
use super::{Orientation, DeleteMe};
use crate::linear_algebra::{UnitVector, Vector};

#[derive(Debug, Clone, Copy)]
pub struct FixedUp {
    orientation: Orientation,
    up: UnitVector<3>,
}

impl Default for FixedUp {
    fn default() -> Self {
        Self {
            orientation: Orientation::default(),
            up: Vector::new([0.0, 1.0, 0.0]).normalize(),
        }
    }
}

impl FixedUp {
    const COS_NO_CONE: f32 = 0.99;
}

impl DeleteMe for FixedUp {
    fn new_forward_up(forward: UnitVector<3>, global_up: UnitVector<3>) -> Self
    where
        Self: Sized,
    {
        let orientation = Orientation::new_forward_up(forward, global_up);
        Self {
            orientation,
            up: global_up,
        }
    }

    #[inline]
    fn view_left(&self) -> UnitVector<3> {
        self.orientation.view_left()
    }

    #[inline]
    fn view_forward(&self) -> UnitVector<3> {
        self.orientation.view_forward()
    }

    #[inline]
    fn view_up(&self) -> UnitVector<3> {
        self.orientation.view_up()
    }
}

impl FixedUp {
    pub fn motion_up(&self) -> UnitVector<3> {
        self.up
    }

    pub fn look_up(&mut self, angle: f32) {
        let fdotu = Vector::dot(self.view_forward().v(), self.up.v());
        if !((angle > 0.0 && fdotu > Self::COS_NO_CONE)
            || (angle < 0.0 && fdotu < -Self::COS_NO_CONE))
        {
            self.orientation.look_up(angle);
        }
    }

    #[inline]
    pub fn look_left(&mut self, angle: f32) {
        let effect = Quaternion::axis_angle(angle, self.up).q();
        let current = self.orientation.into_quaternion().q();

        let new_quaternion = (effect * current).normalize();
        self.orientation = Orientation::from_quaternion(new_quaternion);
    }

    pub fn motion_left(&self) -> UnitVector<3> {
        self.orientation.motion_left()
    }

    pub fn motion_forward(&self) -> UnitVector<3> {
        self.up
            .v()
            .cross(self.view_forward().v().cross(self.up.v()))
            .normalize()
    }

    pub fn reverse_direction(&mut self) {
        self.orientation.reverse_direction();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::geometry::orientation::quaternion;

    #[test]
    fn get_matrix() {
        let orient = FixedUp::new_from_to(
            Vector::from([0.0, 0.0, 0.0]),
            Vector::from([1.0, 1.0, 1.0]),
            Vector::from([0.0, 1.0, 0.0]).normalize(),
        );
        println!("fixed_up: {:?}", orient.as_matrix());
    }
}
