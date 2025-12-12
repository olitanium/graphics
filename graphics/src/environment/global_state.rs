use crate::environment::Event;
use crate::environment::draw::PollResult;
use crate::types::TexDim;

pub trait ApiState: Sized {
    type Error: core::error::Error;
    fn poll<'a>(
        &'a mut self,
        events: Vec<Event<'a>>,
        // default_framebuffer: &'a DefaultFramebuffer,
    ) -> PollResult<'a, Self::Error>;

    fn new(initial_size: (TexDim, TexDim)) -> Result<Self, Self::Error>;
}
