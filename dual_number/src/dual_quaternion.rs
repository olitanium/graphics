
mod unit_dual_quaternion;
use linear_algebra::Vector;
use quaternion::Quaternion;
pub use unit_dual_quaternion::UnitDualQuaternion;

mod sclerp;
pub use sclerp::Sclerp;

use super::DualNumber;

pub type DualQuaternion = DualNumber<Quaternion>;

impl DualQuaternion {
    pub fn new_transpose(scalar: DualNumber<f32>, vector: DualNumber<Vector<3>>) -> Self {
        Self {
            real: Quaternion::from_scalar_vec(scalar.real, vector.real),
            dual: Quaternion::from_scalar_vec(scalar.dual, vector.dual),
        }
    }

    pub fn conjugate(self) -> Self {
        Self {
            real: self.real.conjugate(),
            dual: self.dual.conjugate(),
        }
    }

    pub fn magnitude(self) -> DualNumber<f32> {
        let magnitude = self * self.conjugate();

        assert!(magnitude.real.vector().is_zero());
        assert!(magnitude.dual.vector().is_zero());

        let real_scalar = magnitude.real.scalar();
        let dual_scalar = magnitude.dual.scalar();

        DualNumber {
            real: real_scalar,
            dual: dual_scalar,
        }
    }

    pub fn inverse(self) -> Self {
        // p^{−1} (1 − ε q p^{−1}).
        // = p^{-1} - ε p^{−1} q p^{−1}

        let p = self.real;
        let q = self.dual;

        let p_inverse = p.inverse();

        let real_term = p_inverse;
        let dual_term = -p_inverse * q * p_inverse;

        Self {
            real: real_term,
            dual: dual_term,
        }
    }

    pub fn normalize(self) -> UnitDualQuaternion {
        UnitDualQuaternion::normalize_dual_quaternion(self)
    }
}

#[cfg(test)]
mod test {
    use linear_algebra::Vector;

    use super::*;

    #[test]
    fn test_inverse() {
        let dual_quaternion = DualQuaternion::new(
            Quaternion::from_scalar_vec(1.0, Vector::new([1.0, 1.0, 1.0])),
            Quaternion::from_scalar_vec(2.0, Vector::new([2.0, 2.0, 2.0])),
        );

        let inverse = dual_quaternion.inverse();

        let back = inverse.inverse();

        assert_eq!(dual_quaternion, back)
    }
}
