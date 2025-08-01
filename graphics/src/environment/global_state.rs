use super::Draw;
use crate::environment::Event;
use crate::error::Result;
use crate::framebuffer::DefaultFramebuffer;
use crate::types::TexDim;

pub trait GlobalState: Sized {
    fn poll<'a>(
        &'a mut self,
        events: Vec<Event>,
        default_framebuffer: &'a DefaultFramebuffer,
    ) -> Result<Vec<Box<dyn Draw + 'a>>>;

    fn new(initial_size: (TexDim, TexDim)) -> Result<Self>;
}
