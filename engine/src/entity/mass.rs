#[derive(Debug, Clone, Copy)]
pub struct Mass(pub f32);

impl Default for Mass {
    fn default() -> Self {
        Self(f32::INFINITY)
    }
}
