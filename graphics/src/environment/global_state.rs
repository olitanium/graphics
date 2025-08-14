use super::Draw;
use crate::environment::Event;
use crate::error::ApiResult;
use crate::types::TexDim;

pub trait ApiState: Sized {
    type Error;
    fn poll<'a>(
        &'a mut self,
        events: Vec<Event<'a>>,
        // default_framebuffer: &'a DefaultFramebuffer,
    ) -> ApiResult<Vec<Box<dyn Draw + 'a>>, Self::Error>;

    fn new(initial_size: (TexDim, TexDim)) -> ApiResult<Self, Self::Error>;
}
