//! Defines the `SignedUnitf*` types.
use core::ops;

/// Convenience macro for implemeting `from` various floating point numbers.
macro_rules! fromfloat {
    ($name:ident, $type:ty, $inner:ty) => {
        #[doc = concat!("convert from ", stringify!($type))]
        #[inline]
        pub const fn $name(float: $type) -> Result<Self, $crate::TryFromError<$type>> {
            // quick and dirty version here.
            // float 0.999... == i8::MAX which is 0111 1111;
            // float should be multiplied by the MAX value + 1,
            if float < -1.0 {
                Err($crate::TryFromError::Low(float))
            } else if float > 1.0 {
                Err($crate::TryFromError::High(float))
            } else {
                #[expect(clippy::cast_possible_truncation, reason = "intended behaviour")]
                let inner =
                    (float * const { <$type>::from(<$inner>::MIN.cast_unsigned()) }) as $inner;
                Ok(Self { inner })
            }
        }
    };
}

/// Convenience macro for implemeting `to` various floating point numbers.
macro_rules! tofloat {
    ($name:ident, $other:ty, $type:ty) => {
        #[doc = concat!("convert into ", stringify!($type))]
        #[inline]
        pub const fn $name(self) -> $type {
            if self.inner == 0 {
                return 0.0;
            };
            let sign = self.inner.signum() as $type;

            let inner_abs = self.inner.unsigned_abs() << 1;

            let abs_float = <$other>::new(inner_abs).$name();

            abs_float.copysign(sign)
        }
    };
}

/// Convenience macro to generate multiple sizes of value.
macro_rules! make_sf {
    ($name:ident, $other:ty, $inner:ty, $unsigned:ty) => {
        /// A signed unit floating point with 8 bits.
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
        #[repr(transparent)]
        pub struct $name {
            /// The inner integer storing the value.
            inner: $inner,
        }

        impl $name {
            fromfloat!(from_f32, f32, $inner);

            fromfloat!(from_f64, f64, $inner);

            tofloat!(to_f32, $other, f32);

            tofloat!(to_f64, $other, f64);
        }

        const impl From<$name> for f32 {
            fn from(value: $name) -> Self {
                value.to_f32()
            }
        }

        const impl From<$name> for f64 {
            fn from(value: $name) -> Self {
                value.to_f64()
            }
        }

        const impl TryFrom<f32> for $name {
            type Error = $crate::TryFromError<f32>;

            fn try_from(value: f32) -> Result<Self, Self::Error> {
                Self::from_f32(value)
            }
        }

        const impl TryFrom<f64> for $name {
            type Error = $crate::TryFromError<f64>;

            fn try_from(value: f64) -> Result<Self, Self::Error> {
                Self::from_f64(value)
            }
        }

        impl ops::Mul for $name {
            type Output = Self;

            #[inline]
            fn mul(self, rhs: Self) -> Self::Output {
                let (low, high) = self.inner.carrying_mul(rhs.inner, 0);
                let mut inner = high << 1;
                const MASK: $unsigned = <$inner>::MIN.cast_unsigned();
                if low & MASK > 0 {
                    inner += 1;
                }
                Self::Output { inner }
            }
        }

        const impl ops::Add for $name {
            type Output = Self;

            #[inline]
            /// Add is saturating for clamped floats.
            fn add(self, rhs: Self) -> Self::Output {
                let inner = self.inner.saturating_add(rhs.inner);
                Self { inner }
            }
        }

        const impl ops::Sub for $name {
            type Output = Self;

            #[inline]
            /// Sub is saturating for clamped floats.
            fn sub(self, rhs: Self) -> Self::Output {
                let inner = self.inner.saturating_sub(rhs.inner);
                Self { inner }
            }
        }
    };
}

make_sf!(SignedUnitf8, crate::Unitf8, i8, u8);
make_sf!(SignedUnitf16, crate::Unitf16, i16, u16);

#[expect(clippy::inline_modules, reason = "clippy bug this is a test")]
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn half_times_half() {
        let half = SignedUnitf16::from_f32(0.5).expect("const");

        let quarter = SignedUnitf16::from_f32(0.25).expect("const");

        assert_eq!(half * half, quarter);
    }

    #[test]
    fn round_trip() {
        let half = SignedUnitf16::from_f32(0.5).unwrap();
        let float = half.to_f32();

        assert!((float - 0.5).abs() < 0.00001);
    }

    #[test]
    fn round_trip2() {
        let third = SignedUnitf16::from_f32(-0.33).unwrap();
        let float = third.to_f32();

        assert!((float - (-0.33)).abs() < 0.1);
    }

    #[test]
    fn minus_one() {
        let minus_one = SignedUnitf16::from_f32(-1.0).unwrap();

        assert_eq!(minus_one.inner, i16::MIN);
    }

    #[test]
    fn plus_one() {
        let one = SignedUnitf16::from_f32(1.0).unwrap();

        let nineninenine = SignedUnitf16::from_f32(0.999_999_9).unwrap();

        assert_eq!(nineninenine, one);
    }

    #[test]
    fn rapid_round_trip() {
        let start = 0.617_187_5;
        let value = SignedUnitf16::from_f64(start).unwrap();

        let rapid_float = value.to_f64();

        assert!((start - rapid_float).abs() <= 1.0 / f64::from(u8::MAX));
    }
}
