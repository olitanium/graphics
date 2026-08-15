#[derive(Debug, Clone, Copy)]
pub struct Mass {
    // mass: f32,
    inverse_mass: f32,
}

impl Default for Mass {
    fn default() -> Self {
        Self {
            // mass: f32::INFINITY,
            inverse_mass: 0.0,
        }
    }
}

impl Mass {
    pub const fn new(mass: f32) -> Self {
        Self {
            // mass,
            inverse_mass: 1.0 / mass,
        }
    }

    pub const fn new_inverse(inverse_mass: f32) -> Self {
        Self {
            // mass: 1.0 / inverse_mass,
            inverse_mass,
        }
    }

    pub const fn mass(&self) -> f32 {
        1.0 / self.inverse_mass
    }

    pub const fn inverse_mass(&self) -> f32 {
        self.inverse_mass
    }
}
