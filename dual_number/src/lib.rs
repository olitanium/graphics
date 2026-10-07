#![feature(const_trait_impl)]
#![feature(const_ops)]
#![feature(const_destruct)]
#![feature(const_clone)]

use core::fmt::Debug;
use core::ops::{Add, Mul, Neg, Sub};
use std::marker::Destruct;

mod dual_quaternion;
pub use dual_quaternion::{DualQuaternion, Sclerp, UnitDualQuaternion};

#[cfg(test)]
mod test;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DualNumber<T> {
    pub real: T,
    pub dual: T,
}

impl<T> DualNumber<T> {
    pub const fn new(real: T, dual: T) -> Self {
        Self { real, dual }
    }
}

impl<M, T: Add<M>> Add<DualNumber<M>> for DualNumber<T> {
    type Output = DualNumber<<T as Add<M>>::Output>;

    fn add(self, rhs: DualNumber<M>) -> Self::Output {
        Self::Output {
            real: self.real + rhs.real,
            dual: self.dual + rhs.dual,
        }
    }
}

const impl<M, T> Mul<DualNumber<M>> for DualNumber<T>
where
    Self: [const] Destruct,
    DualNumber<M>: [const] Destruct,
    M: [const] Clone,
    T: [const] Clone + [const] Mul<M>,
    <T as Mul<M>>::Output: [const] Add<Output = <T as Mul<M>>::Output>,
{
    type Output = DualNumber<<T as Mul<M>>::Output>;

    fn mul(self, rhs: DualNumber<M>) -> Self::Output {
        let real = self.real.clone() * rhs.real.clone();
        let dual = self.real * rhs.dual + self.dual * rhs.real;

        Self::Output { real, dual }
    }
}

const impl<T> Mul<f32> for DualNumber<T>
where
    Self: [const] Destruct,
    T: [const] Mul<f32>,
{
    type Output = DualNumber<<T as Mul<f32>>::Output>;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            real: self.real * rhs,
            dual: self.dual * rhs,
        }
    }
}

impl<T> Mul<DualNumber<T>> for f32
where
    DualNumber<T>: Mul<Self>,
{
    type Output = <DualNumber<T> as Mul<Self>>::Output;

    fn mul(self, rhs: DualNumber<T>) -> Self::Output {
        rhs * self
    }
}

impl<M, T: Sub<M>> Sub<DualNumber<M>> for DualNumber<T> {
    type Output = DualNumber<<T as Sub<M>>::Output>;

    fn sub(self, rhs: DualNumber<M>) -> Self::Output {
        Self::Output {
            real: self.real - rhs.real,
            dual: self.dual - rhs.dual,
        }
    }
}

impl<U, T: Neg<Output = U>> Neg for DualNumber<T> {
    type Output = DualNumber<U>;

    fn neg(self) -> Self::Output {
        Self::Output {
            real: -self.real,
            dual: -self.dual,
        }
    }
}

impl DualNumber<f32> {
    #[must_use]
    pub fn sin(self) -> Self {
        // https://math.stackexchange.com/a/1072309
        let Self { real: a, dual: b } = self;

        Self::new(a.sin(), b * a.cos())
    }

    #[must_use]
    pub fn cos(self) -> Self {
        // https://math.stackexchange.com/a/1072309
        let Self { real: a, dual: b } = self;

        Self {
            real: a.cos(),
            dual: -b * a.sin(),
        }
    }

    #[must_use]
    pub fn tan(self) -> Self {
        // https://math.stackexchange.com/a/1072309
        let Self { real: a, dual: b } = self;

        let cos_a = a.cos();
        let secsq_a = 1.0 / (cos_a * cos_a);

        Self {
            real: a.tan(),
            dual: b * secsq_a,
        }
    }
}
