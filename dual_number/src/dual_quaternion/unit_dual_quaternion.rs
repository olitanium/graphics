use utils::rsqrt;

use super::{DualNumber, DualQuaternion};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UnitDualQuaternion(DualQuaternion);

impl UnitDualQuaternion {
    pub const fn normalize_dual_quaternion(dual_quaternion: DualQuaternion) -> Self {
        // https://stackoverflow.com/questions/23174899/properly-normalizing-a-dual-quaternion
        let DualNumber { real: s, dual: t } = dual_quaternion.magnitude_sq();
        let u = rsqrt(s);

        let factor = DualNumber {
            real: u,
            dual: -t * u * u * u * 0.5,
        };

        let normalized = dual_quaternion * factor;

        Self(normalized)
    }

    pub const fn new_normal_unchecked(dual_quaternion: DualQuaternion) -> Self {
        Self(dual_quaternion)
    }

    #[must_use = "returns a new `UnitDualQuaternion`, rather than modifying the original"]
    pub const fn conjugate(self) -> Self {
        Self(self.0.quaternion_conjugate())
    }

    #[must_use = "returns a new `UnitDualQuaternion`, rather than modifying the original"]
    pub const fn inverse(self) -> Self {
        self.conjugate()
    }

    pub const fn u(self) -> DualQuaternion {
        self.0
    }
}

#[cfg(test)]
mod test {}
