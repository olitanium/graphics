use super::{DualNumber, DualQuaternion};

#[derive(Debug, Clone, Copy)]
pub struct UnitDualQuaternion(DualQuaternion);

impl UnitDualQuaternion {
    pub fn normalize_dual_quaternion(dual_quaternion: DualQuaternion) -> Self {
        // https://stackoverflow.com/questions/23174899/properly-normalizing-a-dual-quaternion
        let DualNumber { real: s, dual: t } = dual_quaternion.magnitude();
        let u = 1.0 / s.sqrt();

        let factor = DualNumber {
            real: u,
            dual: t * u * u * u * 0.5,
        };

        let normalized = dual_quaternion * factor;
        Self(normalized)
    }

    pub fn new_normal_unchecked(dual_quaternion: DualQuaternion) -> Self {
        Self(dual_quaternion)
    }

    pub fn conjugate(self) -> Self {
        Self(self.0.conjugate())
    }

    pub fn inverse(self) -> Self {
        self.conjugate()
    }

    pub fn u(self) -> DualQuaternion {
        self.0
    }
}

#[cfg(test)]
mod test {}
