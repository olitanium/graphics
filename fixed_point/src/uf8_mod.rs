#[expect(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct uf8 {
    pub(crate) inner: u8,
}

macro_rules! from_float {
    ($name:ident, $type:ty) => {
        pub const fn $name(float: $type) -> Option<Self> {
            // quick and dirty version here.
            // float 0.999... == u8::MAX which is 1111 1111;
            if float < 0.0 || float > 1.0 {
                return None;
            }
            // if float == 1.0 {
            // return Some(Self { inner: u8::MAX } ) // Unneeded as `f32 as u8` saturates
            // }
            let inner = (float * const { u8::MAX as $type + 1.0 }) as u8;
            Some(Self { inner })
        }
    };
}

macro_rules! to_float {
    ($name:ident, $type:ty, $int:ty) => {
        pub const fn $name(self) -> $type {
            let Some(highest_one) = self.inner.highest_one() else {
                return 0.0;
            };

            let to_left_shift = u8::BITS as $int - highest_one as $int;
            let mantissa = (self.inner << to_left_shift) as $int; // left-shift the first 1 off the end, then extend to a u32
            let exponent = <$type>::MAX_EXP as $int - 1 - to_left_shift; // offset based on IEE754

            let mantissa_mask = mantissa << (<$type>::MANTISSA_DIGITS - 1 - u8::BITS); // mantissa is at LSB
            let exponent_mask = exponent << (<$type>::MANTISSA_DIGITS - 1); // MANTISSS_DIGITS gives length occupied by the mantissa

            let float = mantissa_mask | exponent_mask;

            <$type>::from_bits(float)
        }
    };
}

impl uf8 {
    from_float!(from_f32, f32);

    from_float!(from_f64, f64);

    to_float!(to_f32, f32, u32);

    to_float!(to_f64, f64, u64);
}

impl core::ops::Mul for uf8 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let (_, inner) = self.inner.carrying_mul(rhs.inner, 0);
        Self::Output { inner }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn half_times_half() {
        let half = uf8::from_f32(0.5).unwrap();

        let quarter = const { uf8::from_f32(0.25).unwrap() };

        assert_eq!(half * half, quarter);
    }

    #[test]
    fn round_trip() {
        let half = uf8::from_f32(0.5).unwrap();
        let float = half.to_f32();

        assert!((float - 0.5).abs() < 0.00001);
    }

    #[test]
    fn round_trip2() {
        let third = uf8::from_f32(0.33).unwrap();
        let float = third.to_f32();

        assert!((float - (0.33)).abs() < 0.1);
    }

    #[test]
    fn minus_one() {
        assert!(uf8::from_f32(-1.0).is_none());
    }

    #[test]
    fn plus_one() {
        let one = uf8::from_f32(1.0).unwrap();

        let nineninenine = uf8::from_f32(0.99999999).unwrap();

        assert_eq!(nineninenine, one)
    }

    #[test]
    fn rapid_round_trip() {
        let start = 0.6171875;
        let value = uf8::from_f64(start).unwrap();

        let rapid_float = value.to_f64();

        assert!((start - rapid_float).abs() <= 1.0 / (u8::MAX as f64))
    }
}
