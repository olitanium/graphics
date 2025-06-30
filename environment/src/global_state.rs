pub use glfw::Action::*;

use crate::buffers::DefaultFramebuffer;
use crate::environment::Event;
use crate::modelling::Draw;
use crate::types::TexDim;
use crate::Result;

pub trait GlobalState: Sized {
    fn poll<'a>(
        &'a mut self,
        events: Vec<Event>,
        default_framebuffer: &'a DefaultFramebuffer,
    ) -> Result<Vec<Box<dyn Draw + 'a>>>;

    fn new(initial_size: (TexDim, TexDim)) -> Result<Self>;
}
