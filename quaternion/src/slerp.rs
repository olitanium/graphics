//! Spherical Linear Interpolation.

use super::Quaternion;
use crate::UnitQuaternion;

/// Spherical linear interpolation is a smoother interpolation between two
/// unit quaternions. This is especially useful when the Quaternions represent
/// orientations.
#[derive(Debug, Clone, Copy)]
pub struct Slerp {
    /// Origin quaternion.
    from: Quaternion,
    /// Destination quaternion.
    to: Quaternion,
    /// Pre-calculated field.
    omega: f32,
    /// Pre-calculated field.
    coef: f32,
}

impl Slerp {
    /// Get a new `Slerp` calculator from two `UnitQuaternions`. The `Slerp` can
    /// be configured to go the shortest path or the longest path by setting
    /// `long_way`.
    #[must_use]
    #[inline]
    pub fn new(from: UnitQuaternion, to: UnitQuaternion, long_way: bool) -> Self {
        let from = from.q();
        let mut to = to.q();
        let mut dot = from.dot(to);

        if (long_way && dot.is_sign_positive()) || dot.is_sign_negative() {
            to = -to;
            dot = -dot;
        }

        let omega = dot.acos();
        // let coef = 1.0 / (1.0 - dot * dot).sqrt();
        let coef = 1.0 / f32::mul_add(-dot, dot, 1.0);

        Self {
            from,
            to,
            omega,
            coef,
        }
    }

    /// Use the interpolation to get a `UnitQuaternion` (representing an
    /// orientation).
    #[must_use]
    #[inline]
    pub fn get(self, time: f32) -> UnitQuaternion {
        if self.from == self.to {
            return self.from.normalize();
        }

        let tmp = time * self.omega;
        let s1 = (self.omega - tmp).sin() * self.coef;
        let s2 = tmp.sin() * self.coef;

        let term1 = s1 * self.from;
        let term2 = s2 * self.to;

        (term1 + term2).normalize()
    }
}
