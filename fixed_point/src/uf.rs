//! Defines the `Unitf*` types.
use core::ops;

/// Convenience macro for implemeting `from` various floating point numbers.
macro_rules! from_float {
    ($name:ident, $type:ty, $self_inner:ty) => {
        #[doc = concat!("convert from ", stringify!($type))]
        #[inline]
        pub const fn $name(float: $type) -> Result<Self, $crate::TryFromError<$type>> {
            // quick and dirty version here.
            // float 0.999... == u8::MAX which is 1111 1111;
            if float < 0.0 {
                Err($crate::TryFromError::Low(float))
            } else if float > 1.0 {
                Err($crate::TryFromError::High(float))
            } else {
                #[expect(clippy::cast_possible_truncation, reason = "intended behaviour")]
                #[expect(clippy::cast_sign_loss, reason = "intended behaviour")]
                let inner = (float * const { <$self_inner>::MAX as $type + 1.0 }) as $self_inner;
                Ok(Self { inner })
            }
        }
    };
}

/// Convenience macro for implemeting `to` various floating point numbers.
macro_rules! to_float {
    ($name:ident, $type:ty, $int:ty, $self_inner:ty) => {
        #[doc = concat!("convert into ", stringify!($type))]
        #[inline]
        pub const fn $name(self) -> $type {
            let Some(highest_one) = self.inner.highest_one() else {
                return 0.0;
            };

            let to_left_shift = <$self_inner>::BITS as $int - highest_one as $int;
            let mantissa = (self.inner << to_left_shift) as $int; // left-shift the first 1 off the end, then extend to a u32
            let exponent = <$type>::MAX_EXP as $int - 1 - to_left_shift; // offset based on IEE754

            let mantissa_mask = mantissa << (<$type>::MANTISSA_DIGITS - 1 - <$self_inner>::BITS); // mantissa is at LSB
            let exponent_mask = exponent << (<$type>::MANTISSA_DIGITS - 1); // MANTISSS_DIGITS gives length occupied by the mantissa

            let float = mantissa_mask | exponent_mask;

            <$type>::from_bits(float)
        }
    };
}

/// Convenience macro to generate multiple sizes of value.
macro_rules! make_uf {
    ($name:ident, $inner:ty) => {
        /// An unsigned unit floating point with 8 bits.
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
        #[repr(transparent)]
        pub struct $name {
            pub(crate) inner: $inner,
        }

        impl $name {
            from_float!(from_f32, f32, $inner);

            from_float!(from_f64, f64, $inner);

            to_float!(to_f32, f32, u32, $inner);

            to_float!(to_f64, f64, u64, $inner);

            pub(crate) const fn new(inner: $inner) -> Self {
                Self { inner }
            }
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

        const impl ops::Mul for $name {
            type Output = Self;

            #[inline]
            fn mul(self, rhs: Self) -> Self::Output {
                let (_, inner) = self.inner.carrying_mul(rhs.inner, 0);
                Self { inner }
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

make_uf!(Unitf8, u8);
make_uf!(Unitf16, u16);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn half_times_half() {
        let half = Unitf8::from_f32(0.5).unwrap();

        let quarter = Unitf8::from_f32(0.25).unwrap();

        assert_eq!(half * half, quarter);
    }

    #[test]
    fn round_trip() {
        let half = Unitf8::from_f32(0.5).unwrap();
        let float = half.to_f32();

        assert!((float - 0.5).abs() < 0.00001);
    }

    #[test]
    fn round_trip2() {
        let third = Unitf8::from_f32(0.33).unwrap();
        let float = third.to_f32();

        assert!((float - (0.33)).abs() < 0.01, "{float} ?= 0.33");
    }

    #[test]
    fn minus_one() {
        Unitf8::from_f32(-1.0).unwrap_err();
    }

    #[test]
    fn plus_one() {
        let one = Unitf8::from_f32(1.0).unwrap();

        let nineninenine = Unitf8::from_f32(0.999_999).unwrap();

        assert_eq!(nineninenine, one);
    }

    #[test]
    fn rapid_round_trip() {
        let start = 0.617_187_5;
        let value = Unitf8::from_f64(start).unwrap();

        let rapid_float = value.to_f64();

        assert!((start - rapid_float).abs() <= 1.0 / f64::from(u8::MAX));
    }
}
