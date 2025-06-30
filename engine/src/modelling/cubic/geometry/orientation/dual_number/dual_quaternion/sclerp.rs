use linear_algebra::Vector;

use super::DualQuaternion;
use crate::modelling::cubic::geometry::{DualNumber, Pose, YieldsPose};

#[derive(Debug, Clone, Copy)]
pub struct Sclerp {
    // https://www.xbdev.net/misc_demos/demos/dual_quaternions_beyond/paper.pdf
    from: DualQuaternion,
    v: DualNumber<Vector<3>>,
    theta: DualNumber<f32>,
    duration: f32,
}

impl Sclerp {
    pub fn new(from: Pose, to: Pose, long_way: bool, duration: f32) -> Self {
        let inv_a_times_b = from.into_inner().u().inverse() * to.into_inner().u();

        let int_real = inv_a_times_b.real;
        let int_dual = inv_a_times_b.dual;

        let (wr, vr) = (int_real.scalar(), int_real.vector());
        let (wd, vd) = (int_dual.scalar(), int_dual.vector());

        let theta = 2.0 * wr.acos();
        let d = -2.0 * wd / vr.magnitude();
        let l = vr.normalize().v();
        let m = (vd - 0.5 * l * d * wr) * (1.0 / vr.magnitude());

        let v_f = DualNumber::new(l, m);
        let theta_f = DualNumber::new(theta, d);

        Self {
            from: from.into_inner().u(),
            duration,
            v: v_f,
            theta: theta_f,
        }
    }

    pub fn get(self, time: f32) -> Pose {
        let time = time / self.duration;

        let second_part_scalar = (0.5 * time * self.theta).cos();
        let second_part_vector = self.v * (0.5 * time * self.theta).sin();

        let dual_quaternion = DualQuaternion::new_transpose(second_part_scalar, second_part_vector);
        let output = (self.from * dual_quaternion).normalize();

        Pose::new_from_dual(output)
    }
}

impl YieldsPose for Sclerp {
    type Hint = f32;

    fn get_pose(&self, hint: Self::Hint) -> Pose {
        self.get(hint)
    }
}
