
use super::DualQuaternion;
use crate::UnitDualQuaternion;

#[derive(Debug, Clone, Copy)]
pub struct Sclerp {
    // https://www.xbdev.net/misc_demos/demos/dual_quaternions_beyond/paper.pdf
    // from: DualQuaternion,
    // v: DualNumber<Vector<3>>,
    // theta: DualNumber<f32>,
    // duration: f32,
    from: UnitDualQuaternion,
    to: DualQuaternion,
    duration: f32,
}

impl Sclerp {
    pub fn new(
        from: UnitDualQuaternion,
        to: UnitDualQuaternion,
        duration: f32,
        shortest_path: bool,
    ) -> Self {
        let dot = from.u().real.dot(to.u().dual);
        let to = if (dot < 0.0 && shortest_path) || (dot > 0.0 && !shortest_path) {
            -to.u()
        } else {
            to.u()
        };
        // if from.u() == to.u() {
        // return Self {
        // from: from.u(),
        // v: DualNumber::new(Vector::ZERO, Vector::ZERO),
        // theta: DualNumber::new(0.0, 0.0),
        // duration: 1.0,
        // };
        // }*/
        //
        // let inv_a_times_b = from.u().conjugate() * to;
        //
        // let inv_real = inv_a_times_b.real;
        // let inv_dual = inv_a_times_b.dual;
        //
        // let (wr, vr) = (inv_real.scalar(), inv_real.vector());
        // let (wd, vd) = (inv_dual.scalar(), inv_dual.vector());
        //
        //
        // let angle = 2.0 * wr.acos();
        // let pitch = -2.0 * wd / vr.magnitude();
        // let direction = vr.normalize().v();
        // let moment = (vd - 0.5 * direction * pitch * wr) * (1.0 / vr.magnitude());
        //
        // let v_f = DualNumber::new(direction, moment);
        // let theta_f = DualNumber::new(angle, pitch);
        Self {
            from,
            to,
            duration,
            // duration,
            // v: v_f,
            // theta: theta_f,
        }
    }

    pub fn get(self, time: f32) -> UnitDualQuaternion {
        let time = time / self.duration;

        // let second_part_scalar = (0.5 * time * self.theta).cos();
        // let second_part_vector = self.v * (0.5 * time * self.theta).sin();
        //
        // let dual_quaternion = DualQuaternion::new_transpose(second_part_scalar,
        // second_part_vector); (self.from * dual_quaternion).normalize()
        let from = self.from.u() * (1.0 - time);
        let to = self.to * time;
        (from + to).normalize()
    }
}
