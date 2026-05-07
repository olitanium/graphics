#![feature(never_type)]
#![feature(iter_map_windows)]
#![feature(array_windows)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::fmt::{Debug, Display};

use array_vec::ArrayVec;
use linear_algebra::Vector;

#[derive(Debug, Clone, Copy)]
pub struct BezierSegment<const DIMS: usize, const N: usize>([Vector<DIMS>; N]);

#[derive(Debug, Clone)]
pub struct BezierCurve<const DIMS: usize, const N: usize> {
    segments: Vec<BezierSegment<DIMS, N>>,
}

fn bezier_solve<const DIMS: usize, const N: usize>(
    vecs: &[Vector<DIMS>; N],
    t: f32,
) -> Vector<DIMS> {
    fn iteration<const DIMS: usize, const N: usize>(
        slice: &[Vector<DIMS>],
        t: f32,
    ) -> ArrayVec<Vector<DIMS>, N> {
        slice
            .array_windows()
            .map(|[vec1, vec2]| *vec1 * (1.0 - t) + *vec2 * t)
            .collect()
    }

    if N == 0 {
        Vector::ZERO
    } else {
        let mut out: ArrayVec<Vector<DIMS>, N> = iteration(vecs, t);
        while out.len() > 1 {
            out = iteration(&out, t);
        }
        out[0]
    }
}

impl<const DIMS: usize, const N: usize> BezierSegment<DIMS, N> {
    pub fn new(control: [Vector<DIMS>; N]) -> Self {
        Self(control)
    }

    pub fn get(&self, t: f32) -> Vector<DIMS> {
        bezier_solve::<_, N>(&self.0, t)
    }

    pub fn end(&self) -> Vector<DIMS> {
        self.0.last().copied().unwrap_or_default()
    }

    pub fn start(&self) -> Vector<DIMS> {
        self.0.first().copied().unwrap_or_default()
    }
}

impl<const DIMS: usize, const N: usize> BezierCurve<DIMS, N> {
    pub fn new() -> Self {
        Self { segments: vec![] }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PushError<const DIMS: usize> {
    pub last_point: Vector<DIMS>,
}

impl<const DIMS: usize> Display for PushError<DIMS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
        if let Some(last_point) = self.peek()
            && segment.start() != last_point
        {
            Err(PushError { last_point })
        } else {
            Ok(self.segments.push(segment))
        }
    }

    pub fn peek(&self) -> Option<Vector<DIMS>> {
        self.segments.last().map(|arr| arr.end())
    }

    pub fn get(&self, t: f32) -> Vector<DIMS> {
        let number_of = self.segments.len();

        let t_new = t * number_of as f32;

        self.get_unnormalized(t_new)
    }

    pub fn get_unnormalized(&self, t: f32) -> Vector<DIMS> {
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
        let no_values: Vector<1> = BezierSegment::new([]).get(0.5);
        assert_eq!(no_values, Vector::ZERO)
    }

    #[test]
    fn test_individual() {
        let points = [[0.0, 0.0], [1.0, 1.0]].map(Vector::new);

        let bezier = BezierSegment::new(points);

        for val in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
            let out = bezier.get(val);
            let expected = Vector::new([val, val]);
            assert_eq!(out, expected, "at val {val}");
        }
    }

    #[test]
    fn test_chain() {
        let start = Vector::new([0.0, 0.0]);
        let finish = Vector::new([0.0, 0.0]);

        let segment_one = BezierSegment::new([start, finish]);
        let segment_two = BezierSegment::new([finish, start]);

        let mut bezier = BezierCurve::new();
        bezier.push(segment_one).unwrap();
        bezier.push(segment_two).unwrap();

        assert_eq!(bezier.get(0.0), start);
        assert_eq!(bezier.get(0.5), finish);
        assert_eq!(bezier.get(1.0), start);

        assert_eq!(bezier.get_unnormalized(0.0), start);
        assert_eq!(bezier.get_unnormalized(1.0), finish);
        assert_eq!(bezier.get_unnormalized(2.0), start);
    }
}
