#![feature(const_trait_impl)]
macro_rules! make_nearly_eq {
    ($name:ident : $Float:ty) => {
        /// Calculate whether two floats are "nearly equal" -> numbers which are
        /// proportionally between `epsilon` of one another are equal, and any number
        /// between `[-treat_lower_as_zero, treat_lower_as_zero]` is treated as 0.0 and
        /// compared appropriately.
        #[inline]
        const fn $name(
            lhs: $Float,
            rhs: $Float,
            epsilon: $Float,
            treat_lower_as_zero: $Float,
        ) -> bool {
            if lhs == rhs {
                return true;
            }

            let diff = (lhs - rhs).abs();
            let norm = <$Float>::min(lhs.abs() + rhs.abs(), <$Float>::MAX);

            diff < <$Float>::max(treat_lower_as_zero, epsilon * norm)
        }
    };
}

make_nearly_eq!(nearly_eq_f32: f32);
make_nearly_eq!(nearly_eq_f64: f64);

/// Calculate whether two floats are "nearly equal" -> numbers which are
/// proportionally between `proportion` of one another are equal, and any number
/// between `[-smallest, smallest]` is treated as 0.0 and
/// compared appropriately.
pub const trait NearlyEq {
    const SMALL_VALUE: Self;
    fn nearly_eq(&self, other: &Self, proportion: Self, smallest: Self) -> bool;
}

const impl NearlyEq for f32 {
    const SMALL_VALUE: Self = 0.001;

    fn nearly_eq(&self, other: &Self, proportion: Self, smallest: Self) -> bool {
        nearly_eq_f32(*self, *other, proportion, smallest)
    }
}

const impl NearlyEq for f64 {
    const SMALL_VALUE: Self = 0.001;

    fn nearly_eq(&self, other: &Self, proportion: Self, smallest: Self) -> bool {
        nearly_eq_f64(*self, *other, proportion, smallest)
    }
}

#[cfg(test)]
mod test {
    use crate::nearly_eq_f32;

    fn nearly_eq_chosen(lhs: f32, rhs: f32) -> bool {
        crate::nearly_eq_f32(lhs, rhs, 0.00001, 0.00001)
    }

    /// Regular large numbers - generally not problematic
    #[test]
    fn big() {
        assert!(nearly_eq_chosen(1000000.0, 1000001.0));
        assert!(nearly_eq_chosen(1000001.0, 1000000.0));
        assert!(!nearly_eq_chosen(10000.0, 10001.0));
        assert!(!nearly_eq_chosen(10001.0, 10000.0));
    }

    /// Negative large numbers
    #[test]
    fn big_neg() {
        assert!(nearly_eq_chosen(-1000000.0, -1000001.0));
        assert!(nearly_eq_chosen(-1000001.0, -1000000.0));
        assert!(!nearly_eq_chosen(-10000.0, -10001.0));
        assert!(!nearly_eq_chosen(-10001.0, -10000.0));
    }

    /// Numbers around 1
    #[test]
    fn mid() {
        assert!(nearly_eq_chosen(1.0000001, 1.0000002));
        assert!(nearly_eq_chosen(1.0000002, 1.0000001));
        assert!(!nearly_eq_chosen(1.0002, 1.0001));
        assert!(!nearly_eq_chosen(1.0001, 1.0002));
    }

    /// Numbers around -1
    #[test]
    fn mid_neg() {
        assert!(nearly_eq_chosen(-1.000001, -1.000002));
        assert!(nearly_eq_chosen(-1.000002, -1.000001));
        assert!(!nearly_eq_chosen(-1.0001, -1.0002));
        assert!(!nearly_eq_chosen(-1.0002, -1.0001));
    }

    /// Numbers between 1 and 0
    #[test]
    fn small() {
        assert!(nearly_eq_chosen(0.000000001000001, 0.000000001000002));
        assert!(nearly_eq_chosen(0.000000001000002, 0.000000001000001));
        assert!(nearly_eq_chosen(0.000000000001002, 0.000000000001001));
        assert!(nearly_eq_chosen(0.000000000001001, 0.000000000001002));
    }

    /// Numbers between -1 and 0
    #[test]
    fn small_neg() {
        assert!(nearly_eq_chosen(-0.000000001000001, -0.000000001000002));
        assert!(nearly_eq_chosen(-0.000000001000002, -0.000000001000001));
        assert!(nearly_eq_chosen(-0.000000000001002, -0.000000000001001));
        assert!(nearly_eq_chosen(-0.000000000001001, -0.000000000001002));
    }

    /// Small differences away from zero
    #[test]
    fn small_diffs() {
        assert!(nearly_eq_chosen(0.3, 0.30000003));
        assert!(nearly_eq_chosen(-0.3, -0.30000003));
    }

    /// Comparisons involving zero
    #[test]
    fn zero() {
        assert!(nearly_eq_chosen(0.0, 0.0));
        assert!(nearly_eq_chosen(0.0, -0.0));
        assert!(nearly_eq_chosen(-0.0, -0.0));
        assert!(nearly_eq_chosen(0.00000001, 0.0));
        assert!(nearly_eq_chosen(0.0, 0.00000001));
        assert!(nearly_eq_chosen(-0.00000001, 0.0));
        assert!(nearly_eq_chosen(0.0, -0.00000001));

        assert!(nearly_eq_f32(0.0, 1e-40, 0.01, 0.01));
        assert!(nearly_eq_f32(1e-40, 0.0, 0.01, 0.01));
        assert!(nearly_eq_f32(1e-40, 0.0, 0.000001, 0.000001));
        assert!(nearly_eq_f32(0.0, 1e-40, 0.000001, 0.000001));

        assert!(nearly_eq_f32(0.0, -1e-40, 0.1, 0.1));
        assert!(nearly_eq_f32(-1e-40, 0.0, 0.1, 0.1));
        assert!(nearly_eq_f32(-1e-40, 0.0, 0.00000001, 0.0000001));
        assert!(nearly_eq_f32(0.0, -1e-40, 0.00000001, 0.0000001));
    }

    /// Comparisons involving extreme values (overflow potential)
    #[test]
    fn extreme_max() {
        assert!(nearly_eq_chosen(f32::MAX, f32::MAX));
        assert!(!nearly_eq_chosen(f32::MAX, -f32::MAX));
        assert!(!nearly_eq_chosen(-f32::MAX, f32::MAX));
        assert!(!nearly_eq_chosen(f32::MAX, f32::MAX / 2.0));
        assert!(!nearly_eq_chosen(f32::MAX, -f32::MAX / 2.0));
        assert!(!nearly_eq_chosen(-f32::MAX, f32::MAX / 2.0));
    }

    /// Comparisons involving infinities
    #[test]
    fn infinities() {
        assert!(nearly_eq_chosen(f32::INFINITY, f32::INFINITY));
        assert!(nearly_eq_chosen(f32::NEG_INFINITY, f32::NEG_INFINITY));
        assert!(!nearly_eq_chosen(f32::NEG_INFINITY, f32::INFINITY));
        assert!(!nearly_eq_chosen(f32::INFINITY, f32::MAX));
        assert!(!nearly_eq_chosen(f32::NEG_INFINITY, -f32::MAX));
    }

    /// Comparisons involving::NAN values
    #[test]
    fn nan() {
        assert!(!nearly_eq_chosen(f32::NAN, f32::NAN));
        assert!(!nearly_eq_chosen(f32::NAN, 0.0));
        assert!(!nearly_eq_chosen(-0.0, f32::NAN));
        assert!(!nearly_eq_chosen(f32::NAN, -0.0));
        assert!(!nearly_eq_chosen(0.0, f32::NAN));
        assert!(!nearly_eq_chosen(f32::NAN, f32::INFINITY));
        assert!(!nearly_eq_chosen(f32::INFINITY, f32::NAN));
        assert!(!nearly_eq_chosen(f32::NAN, f32::NEG_INFINITY));
        assert!(!nearly_eq_chosen(f32::NEG_INFINITY, f32::NAN));
        assert!(!nearly_eq_chosen(f32::NAN, f32::MAX));
        assert!(!nearly_eq_chosen(f32::MAX, f32::NAN));
        assert!(!nearly_eq_chosen(f32::NAN, -f32::MAX));
        assert!(!nearly_eq_chosen(-f32::MAX, f32::NAN));
        assert!(!nearly_eq_chosen(f32::NAN, f32::MIN_POSITIVE));
        assert!(!nearly_eq_chosen(f32::MIN_POSITIVE, f32::NAN));
        assert!(!nearly_eq_chosen(f32::NAN, -f32::MIN_POSITIVE));
        assert!(!nearly_eq_chosen(-f32::MIN_POSITIVE, f32::NAN));
    }

    /// Comparisons of numbers on opposite sides of 0
    #[test]
    fn opposite() {
        assert!(!nearly_eq_chosen(1.000000001, -1.0));
        assert!(!nearly_eq_chosen(-1.0, 1.000000001));
        assert!(!nearly_eq_chosen(-1.000000001, 1.0));
        assert!(!nearly_eq_chosen(1.0, -1.000000001));
        assert!(nearly_eq_chosen(
            10.0 * f32::MIN_POSITIVE,
            10.0 * -f32::MIN_POSITIVE
        ));
        assert!(nearly_eq_chosen(
            10000.0 * f32::MIN_POSITIVE,
            10000.0 * -f32::MIN_POSITIVE
        ));
    }

    /// The really tricky part - comparisons of numbers very close to zero.
    #[test]
    fn ulp() {
        assert!(nearly_eq_chosen(f32::MIN_POSITIVE, f32::MIN_POSITIVE));
        assert!(nearly_eq_chosen(f32::MIN_POSITIVE, -f32::MIN_POSITIVE));
        assert!(nearly_eq_chosen(-f32::MIN_POSITIVE, f32::MIN_POSITIVE));
        assert!(nearly_eq_chosen(f32::MIN_POSITIVE, 0.0));
        assert!(nearly_eq_chosen(0.0, f32::MIN_POSITIVE));
        assert!(nearly_eq_chosen(-f32::MIN_POSITIVE, 0.0));
        assert!(nearly_eq_chosen(0.0, -f32::MIN_POSITIVE));

        assert!(nearly_eq_chosen(0.000000001, -f32::MIN_POSITIVE));
        assert!(nearly_eq_chosen(0.000000001, f32::MIN_POSITIVE));
        assert!(nearly_eq_chosen(f32::MIN_POSITIVE, 0.000000001));
        assert!(nearly_eq_chosen(-f32::MIN_POSITIVE, 0.000000001));
    }
}
