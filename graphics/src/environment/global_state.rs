use core::error::Error;

use crate::Draw;
use crate::environment::Event;
use crate::environment::draw::PollResult;
use crate::types::TexDim;

pub trait ApiState: Sized {
    type Error: Error;
    type DrawOutput<'a>: Draw + 'a
    where
        Self: 'a;

    fn poll<'a>(
        &'a mut self,
        events: Vec<Event<'a>>,
    ) -> PollResult<Self::DrawOutput<'a>, Self::Error>;

    fn new(initial_size: (TexDim, TexDim)) -> Result<Self, Self::Error>;
}
