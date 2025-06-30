use std::ops::{Add, Mul};

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
    pub fn new(start: T, finish: T, duration: f32) -> Self {
        Self {
            start,
            finish,
            duration,
        }
    }

    pub fn get(self, time: f32) -> T {
        if time == self.duration {
            return self.finish;
        }

        let proportion = (time / self.duration) % 1.0;
        let oneminusproportion = 1.0 - proportion;

        (self.start * oneminusproportion) + (self.finish * proportion)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn float_eq(a: f32, b: f32) -> bool {
        const PRECISION: f32 = 0.01;
        (a - b).abs() <= PRECISION
    }

    fn assert_float_eq(a: f32, b: f32) {
        assert!(float_eq(a, b))
    }

    #[test]
    fn test_prop() {
        const DURATION: f32 = 10.0;
        let lerp = Lerp::new(0.0, 1.0, DURATION);

        let test_outputs = [0.0, 0.1, 0.2, 0.3, 1.0];
        let test_inpus = test_outputs.map(|x| x * DURATION);

        for (input, output) in std::iter::zip(test_inpus, test_outputs) {
            assert_float_eq(lerp.get(input), output);
        }
    }
}
