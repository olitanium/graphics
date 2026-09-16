use core::fmt::{Debug, Display};

use array_vec::ArrayVec;
use linear_algebra::Vector;
use utils::error_boilerplate;

#[derive(Debug, Clone, Copy)]
pub struct BezierSegment<const DIMS: usize, const N: usize>([Vector<DIMS>; N]);

#[derive(Debug, Clone)]
pub struct BezierCurve<const DIMS: usize, const N: usize> {
    segments: Vec<BezierSegment<DIMS, N>>,
}

impl<const DIMS: usize, const N: usize> BezierSegment<DIMS, N> {
    #[must_use]
    pub const fn new(control: [Vector<DIMS>; N]) -> Self {
        Self(control)
    }
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum GetError {
    ZeroDimension,
}

error_boilerplate!(GetError);

impl<const DIMS: usize, const N: usize> BezierSegment<DIMS, N> {
    pub fn get(&self, t: f32) -> Result<Vector<DIMS>, GetError> {
        let mut out: ArrayVec<Vector<DIMS>, N> = ArrayVec::from_array(self.0);
        // while out.len() > 1 {
        // out = out
        // .array_windows()
        // .copied()
        // .map(|[vec1, vec2]| vec1 * (1.0 - t) + vec2 * t)
        // .collect()
        // }
        //
        // out.get(0).copied().ok_or(GetError::ZeroDimension)

        loop {
            match out.as_slice() {
                &[] => return Err(GetError::ZeroDimension),
                &[out] => return Ok(out),
                other => {
                    out = other
                        .array_windows()
                        .copied()
                        .map(|[vec1, vec2]| vec1 * (1.0 - t) + vec2 * t)
                        .collect();
                }
            }
        }
    }

    pub fn end(&self) -> Result<Vector<DIMS>, GetError> {
        self.0.last().copied().ok_or(GetError::ZeroDimension)
    }

    pub fn start(&self) -> Result<Vector<DIMS>, GetError> {
        self.0.first().copied().ok_or(GetError::ZeroDimension)
    }
}

impl<const DIMS: usize, const N: usize> Default for BezierCurve<DIMS, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const DIMS: usize, const N: usize> BezierCurve<DIMS, N> {
    #[must_use]
    pub const fn new() -> Self {
        Self { segments: vec![] }
    }
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum PushError<const DIMS: usize> {
    IncompatibleStartAndFinishPoints { end_of_previous: Vector<DIMS> },
    ZeroDimension,
}
// Error boilerplate
impl<const DIMS: usize> Display for PushError<DIMS> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl<const DIMS: usize> core::error::Error for PushError<DIMS> {}

impl<const DIMS: usize> From<!> for PushError<DIMS> {
    fn from(value: !) -> Self {
        value
    }
}

impl<const DIMS: usize, const N: usize> BezierCurve<DIMS, N> {
    pub fn push(&mut self, segment: BezierSegment<DIMS, N>) -> Result<(), PushError<DIMS>> {
        match (segment.start(), self.peek()) {
            (Err(GetError::ZeroDimension), _) => Err(PushError::ZeroDimension),
            (_, Err(PeekError::ZeroDimension)) => Err(PushError::ZeroDimension),
            (Ok(start), Ok(peek)) if start != peek => {
                Err(PushError::IncompatibleStartAndFinishPoints {
                    end_of_previous: peek,
                })
            }
            (Ok(_), Ok(_) | Err(PeekError::NoSegment)) => {
                self.segments.push(segment);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PeekError {
    NoSegment,
    ZeroDimension,
}

error_boilerplate!(PeekError);

impl<const DIMS: usize, const N: usize> BezierCurve<DIMS, N> {
    pub fn peek(&self) -> Result<Vector<DIMS>, PeekError> {
        let Some(segment) = self.segments.last() else {
            return Err(PeekError::NoSegment);
        };
        let Ok(end) = segment.end() else {
            return Err(PeekError::ZeroDimension);
        };
        Ok(end)
    }
}

impl<const DIMS: usize, const N: usize> BezierCurve<DIMS, N> {
    pub fn get(&self, t: f32) -> Result<Vector<DIMS>, GetError> {
        let number_of = self.segments.len();

        let t_new = t * number_of as f32;

        self.get_unnormalized(t_new)
    }

    pub fn get_unnormalized(&self, t: f32) -> Result<Vector<DIMS>, GetError> {
        let upper = self.segments.len();

        let (choice, part) = match t {
            t @ ..0.0 => (0, t),

            t if t >= upper as f32 => (upper - 1, t - (upper - 1) as f32),
            t => {
                let floor = t.floor();
                let choice = floor as usize;
                let part = t - floor;

                (choice, part)
            }
        };

        let segment = &self.segments[choice];

        segment.get(part)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        BezierSegment::<3, _>::new([]).get(0.5).unwrap_err();
    }

    #[test]
    fn test_individual() {
        let points = [[0.0, 0.0], [1.0, 1.0]].map(Vector::new);

        let bezier = BezierSegment::new(points);

        for val in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
            let out = bezier.get(val).unwrap();
            let expected = Vector::new([val, val]);
            assert_eq!(out, expected, "at val {val}");
        }
    }

    #[test]
    fn test_chain() {
        let start = Vector::new([1.0, 0.0]);
        let finish = Vector::new([0.0, 1.0]);

        let segment_one = BezierSegment::new([start, finish]);
        let segment_two = BezierSegment::new([finish, start]);

        let mut bezier = BezierCurve::new();
        bezier.push(segment_one).unwrap();
        bezier.push(segment_two).unwrap();

        assert_eq!(bezier.get(0.0).unwrap(), start);
        assert_eq!(bezier.get(0.5).unwrap(), finish);
        assert_eq!(bezier.get(1.0).unwrap(), start);

        assert_eq!(bezier.get_unnormalized(0.0).unwrap(), start);
        assert_eq!(bezier.get_unnormalized(1.0).unwrap(), finish);
        assert_eq!(bezier.get_unnormalized(2.0).unwrap(), start);
    }
}
