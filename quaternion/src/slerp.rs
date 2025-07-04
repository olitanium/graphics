use crate::UnitQuaternion;

use super::Quaternion;


#[derive(Debug, Clone, Copy)]
pub struct Slerp {
    from: Quaternion,
    to: Quaternion,
    duration: f32,
    omega: f32,
    coef: f32,
}

impl Slerp {
    pub fn new(from: UnitQuaternion, to: UnitQuaternion, duration: f32, long_way: bool) -> Self {
        let from = from.q();
        let mut to = to.q();
        let mut dot = from.dot(to);

        if long_way {
            if dot.is_sign_positive() {
                to = -to;
                dot = -dot;
            }
        } else {
            if dot.is_sign_negative() {
                to = -to;
                dot = -dot;
            }
        }

        let omega = dot.acos();
        let coef = 1.0 / (1.0 - dot * dot).sqrt();

        Self {
            from,
            to,
            duration,
            omega,
            coef,
        }
    }

    pub fn get(self, time: f32) -> UnitQuaternion {
        if self.from == self.to {
            self.from.normalize();
        }

        let time = (time / self.duration) % 1.0;

        let tmp = time * self.omega;
        let s1 = (self.omega - tmp).sin() * self.coef;
        let s2 = tmp.sin() * self.coef;

        let term1 = s1 * self.from;
        let term2 = s2 * self.to;
        let quaternion = (term1 + term2).normalize();
    
        quaternion
    }
}
