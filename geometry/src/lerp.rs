use core::ops::{Add, Mul};

#[derive(Debug, Copy, Clone)]
pub struct Lerp<T> {
    start: T,
    finish: T,
    duration: f32,
}

impl<T> Lerp<T>
where
    T: Add<Output = T>,
    T: Mul<f32, Output = T>,
{
    pub const fn new(start: T, finish: T, duration: f32) -> Self {
        Self {
            start,
            finish,
            duration,
        }
    }

    pub fn get(self, time: f32) -> T {
        let proportion = (time / self.duration).rem_euclid(1.0);
        let oneminusproportion = 1.0 - proportion;

        (self.start * oneminusproportion) + (self.finish * proportion)
    }
}

#[cfg(test)]
mod test {
    use core::iter;

    use float_eq::NearlyEq;

    use super::*;

    #[test]
    fn prop() {
        const DURATION: f32 = 10.0;
        let lerp = Lerp::new(0.0, 1.0, DURATION);

        let test_outputs = [0.0, 0.1, 0.2, 0.3, 1.0];
        let test_inpus = test_outputs.map(|x| x * DURATION);

        for (input, output) in iter::zip(test_inpus, test_outputs) {
            NearlyEq::nearly_eq(&lerp.get(input), &output, 0.01, 0.01);
        }
    }
}
