use crate::uf8;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct sf8 {
    pub(crate) inner: i8,
}
macro_rules! fromfloat {
    ($name:ident, $type:ty) => {
        pub const fn $name(float: $type) -> Option<Self> {
            // quick and dirty version here.
            // float 0.999... == i8::MAX which is 0111 1111;
            // float should be multiplied by the MAX value + 1,
            if float < -1.0 || float > 1.0 {
                return None;
            }
            // if float == 1.0 {
            // return Some(Self { inner: i8::MAX } ) // Unneeded as f32 as i8 saturates
            // }
            let inner = (float * const { i8::MIN.cast_unsigned() as $type }) as i8;
            Some(Self { inner })
        }
    };
}

macro_rules! tofloat {
    ($name:ident, $type:ty) => {
        pub const fn $name(self) -> $type {
            if self.inner == 0 {
                return 0.0;
            };
            let sign = self.inner.signum() as $type;

            let inner_abs = self.inner.unsigned_abs() << 1;

            let abs_float = uf8 { inner: inner_abs }.$name();

            abs_float.copysign(sign)
        }
    };
}

impl sf8 {
    fromfloat!(from_f32, f32);

    fromfloat!(from_f64, f64);

    tofloat!(to_f32, f32);

    tofloat!(to_f64, f64);
}

impl core::ops::Mul for sf8 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let (low, high) = self.inner.carrying_mul(rhs.inner, 0);
        let mut inner = high << 1;
        const MASK: u8 = i8::MIN.cast_unsigned();
        if low & MASK > 0 {
            inner += 1
        }
        Self::Output { inner }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn half_times_half() {
        let half = sf8::from_f32(0.5).expect("const");

        let quarter = sf8::from_f32(0.25).expect("const");

        assert_eq!(half * half, quarter);
    }

    #[test]
    fn round_trip() {
        let half = sf8::from_f32(0.5).unwrap();
        let float = half.to_f32();

        assert!((float - 0.5).abs() < 0.00001);
    }

    #[test]
    fn round_trip2() {
        let third = sf8::from_f32(-0.33).unwrap();
        let float = third.to_f32();

        dbg!(float);
        assert!((float - (-0.33)).abs() < 0.1);
    }

    #[test]
    fn minus_one() {
        let minus_one = sf8::from_f32(-1.0).unwrap();

        assert_eq!(minus_one.inner, i8::MIN);
    }

    #[test]
    fn plus_one() {
        let one = sf8::from_f32(1.0).unwrap();

        let nineninenine = sf8::from_f32(0.99999999).unwrap();

        assert_eq!(nineninenine, one)
    }

    #[test]
    fn rapid_round_trip() {
        let start = 0.6171875;
        let value = sf8::from_f64(start).unwrap();

        let rapid_float = value.to_f64();

        assert!((start - rapid_float).abs() <= 1.0 / (u8::MAX as f64))
    }
}
