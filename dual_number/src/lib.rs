use std::ops::{Add, Mul, Neg, Sub};

mod dual_quaternion;
pub use dual_quaternion::{DualQuaternion, Sclerp, UnitDualQuaternion};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DualNumber<Z> {
    pub real: Z,
    pub dual: Z,
}

impl<T> DualNumber<T> {
    pub fn new(real: T, dual: T) -> Self {
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

impl<M, T> Mul<DualNumber<M>> for DualNumber<T>
where
    M: Clone,
    T: Clone,
    T: Mul<M>,
    <T as Mul<M>>::Output: Add<Output = <T as Mul<M>>::Output>,
{
    type Output = DualNumber<<T as Mul<M>>::Output>;

    fn mul(self, rhs: DualNumber<M>) -> Self::Output {
        let real = self.real.clone() * rhs.real.clone();
        let dual = self.real * rhs.dual + self.dual * rhs.real;
        Self::Output { real, dual }
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

impl<T: Neg<Output = T>> Neg for DualNumber<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            real: -self.real,
            dual: -self.dual,
        }
    }
}

impl<T> Mul<DualNumber<T>> for f32
where
    DualNumber<T>: Mul<f32>,
{
    type Output = <DualNumber<T> as Mul<f32>>::Output;

    fn mul(self, rhs: DualNumber<T>) -> Self::Output {
        rhs * self
    }
}

impl Mul<f32> for DualNumber<f32> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            real: self.real * rhs,
            dual: self.dual * rhs,
        }
    }
}

impl DualNumber<f32> {
    pub fn sin(self) -> Self {
        // https://math.stackexchange.com/a/1072309
        let Self { real: a, dual: b } = self;

        Self::new(a.sin(), b * a.cos())
    }

    pub fn cos(self) -> Self {
        // https://math.stackexchange.com/a/1072309
        let Self { real: a, dual: b } = self;

        Self {
            real: a.cos(),
            dual: -b * a.sin(),
        }
    }

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
