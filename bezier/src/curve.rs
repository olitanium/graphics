use core::{error, fmt};

use linear_algebra::Vector;
use utils::error_boilerplate;

use crate::BezierSegment;
use crate::segment::GetError as SegmentGetError;

#[derive(Debug, Clone)]
pub struct BezierCurve<const DIMS: usize, const N: usize> {
    segments: Vec<BezierSegment<DIMS, N>>,
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
impl<const DIMS: usize> fmt::Display for PushError<DIMS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl<const DIMS: usize> error::Error for PushError<DIMS> {}

impl<const DIMS: usize> From<!> for PushError<DIMS> {
    fn from(value: !) -> Self {
        value
    }
}

impl<const DIMS: usize, const N: usize> BezierCurve<DIMS, N> {
    pub fn push(&mut self, segment: BezierSegment<DIMS, N>) -> Result<(), PushError<DIMS>> {
        match (segment.start(), self.peek()) {
            (Err(SegmentGetError::ZeroDimension), _) => Err(PushError::ZeroDimension),
            (_, Err(GetError::Segment(SegmentGetError::ZeroDimension))) => {
                Err(PushError::ZeroDimension)
            }
            (Ok(start), Ok(peek)) if start != peek => {
                Err(PushError::IncompatibleStartAndFinishPoints {
                    end_of_previous: peek,
                })
            }
            (Ok(_), Ok(_) | Err(GetError::NoSegment)) => {
                self.segments.push(segment);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GetError {
    NoSegment,
    Segment(SegmentGetError),
}

error_boilerplate!(GetError);

impl<const DIMS: usize, const N: usize> BezierCurve<DIMS, N> {
    pub fn peek(&self) -> Result<Vector<DIMS>, GetError> {
        let Some(segment) = self.segments.last() else {
            return Err(GetError::NoSegment);
        };
        segment.end().map_err(GetError::Segment)
    }
}

impl<const DIMS: usize, const N: usize> BezierCurve<DIMS, N> {
    pub fn get(&self, t: f32) -> Result<Vector<DIMS>, GetError> {
        let number_of = self.segments.len();

        #[expect(
            clippy::cast_precision_loss,
            reason = "len need not actually be usize, will unlikely be a bezier curve with 2^24 \
                      elements"
        )]
        let t_new = t * number_of as f32;

        self.get_unnormalized(t_new)
    }

    pub fn get_unnormalized(&self, t: f32) -> Result<Vector<DIMS>, GetError> {
        let upper = self.segments.len();

        #[expect(clippy::shadow_same, reason = "better than using `_`")]
        let (choice, part) = match t {
            t @ ..0.0 => (0, t),

            #[expect(
                clippy::cast_precision_loss,
                reason = "lengths will likely be small enough"
            )]
            t if t >= upper as f32 => (upper - 1, t - (upper - 1) as f32),
            t => {
                let floor = t.floor();
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "lengths will likely be small enough"
                )]
                #[expect(
                    clippy::cast_sign_loss,
                    reason = "floor came from a usize so will be positive"
                )]
                let choice = floor as usize;
                let part = t - floor;

                (choice, part)
            }
        };

        // new version

        #[expect(clippy::cast_possible_truncation, reason = "intended behaviour")]
        #[expect(clippy::cast_sign_loss, reason = "intended behaviour")]
        let floor_choice_clamp = t as usize;
        let new_choice = usize::min(floor_choice_clamp, self.segments.len());
        #[expect(
            clippy::cast_precision_loss,
            reason = "new_choice came from f32 originally"
        )]
        let new_part = t - new_choice as f32;

        debug_assert_eq!(new_choice, choice, "testing new, simpler, method");
        debug_assert_eq!(new_part, part, "testing new, simpler, method");

        // get segment

        let segment = self.segments.get(new_choice).ok_or(GetError::NoSegment)?;

        segment.get(part).map_err(GetError::Segment)
    }
}
