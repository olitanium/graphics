use array_vec::ArrayVec;
use linear_algebra::Vector;
use utils::error_boilerplate;

#[derive(Debug, Clone, Copy)]
pub struct BezierSegment<const DIMS: usize, const N: usize>([Vector<DIMS>; N]);

impl<const DIMS: usize, const N: usize> BezierSegment<DIMS, N> {
    #[must_use]
    pub const fn new(control: [Vector<DIMS>; N]) -> Self {
        Self(control)
    }
}

#[derive(Debug, Clone, Copy)]
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
