use std::ops;

use linear_algebra::{UnitVector, Vector};
pub use unit_quaternion::UnitQuaternion;

mod slerp;
mod unit_quaternion;
pub use slerp::Slerp;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Quaternion {
    real: f32,
    vector: Vector<3>,
}

impl Quaternion {
    pub const fn new(real: f32, i: f32, j: f32, k: f32) -> Self {
        Self {
            real,
            vector: Vector::new([i, j, k]),
        }
    }

    pub const fn from_scalar_vec(scalar: f32, vector: Vector<3>) -> Self {
        Self {
            real: scalar,
            vector,
        }
    }

    pub fn is_zero(self) -> bool {
        const EPSILON: f32 = 0.001;

        let real_same = self.real.abs() <= EPSILON;
        let vec_same = self.vector.is_zero();

        real_same && vec_same
    }

    pub fn dot(self, other: Self) -> f32 {
        self.real.mul_add(other.real, self.vector.dot(other.vector))
    }

    pub fn vector(self) -> Vector<3> {
        self.vector
    }

    pub fn scalar(self) -> f32 {
        self.real
    }

    pub fn axis_angle(angle: f32, axis: UnitVector<3>) -> UnitQuaternion {
        let sin = (angle / 2.0).sin();
        let cos = (angle / 2.0).cos();
        let vector = axis.v().scale(sin);
        let non_normal = Self { real: cos, vector }; // Should already be normal
        non_normal.normalize()
    }

    pub fn conjugate(mut self) -> Self {
        self.vector = -self.vector;
        self
    }

    pub fn from_vec(vector: Vector<3>) -> Self {
        Self { real: 0.0, vector }
    }

    pub const fn identity() -> Self {
        Self {
            real: 1.0,
            vector: Vector::new_zero(),
        }
    }

    pub fn scale(self, scalar: f32) -> Self {
        Self {
            real: self.real * scalar,
            vector: self.vector * scalar,
        }
    }

    pub fn inverse(self) -> Self {
        self.conjugate().scale(1.0 / self.magnitude_sq())
    }

    pub fn magnitude_sq(self) -> f32 {
        self.real * self.real + self.vector.magnitude_sq()
    }

    pub fn magnitude(self) -> f32 {
        self.magnitude_sq().sqrt()
    }

    pub fn normalize(self) -> UnitQuaternion {
        UnitQuaternion::normalize_quaternion(self)
    }
}

impl ops::Add for Quaternion {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            real: self.real + rhs.real,
            vector: self.vector + rhs.vector,
        }
    }
}

impl ops::Sub for Quaternion {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            real: self.real - rhs.real,
            vector: self.vector - rhs.vector,
        }
    }
}

impl ops::Mul for Quaternion {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            real: self.real * rhs.real - self.vector.dot(rhs.vector),
            vector: rhs.vector.scale(self.real)
                + self.vector.scale(rhs.real)
                + self.vector.cross(rhs.vector),
        }
    }
}

impl ops::Mul<f32> for Quaternion {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        self.scale(rhs)
    }
}

impl ops::Mul<Quaternion> for f32 {
    type Output = Quaternion;

    fn mul(self, rhs: Quaternion) -> Self::Output {
        rhs * self
    }
}

impl ops::Neg for Quaternion {
    type Output = Self;

    fn neg(self) -> Self::Output {
        let vector = -self.vector;
        let real = -self.real;

        Self { real, vector }
    }
}
