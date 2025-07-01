use crate::buffers::framebuffer::FramebufferContext;
use crate::shader_program::ShaderProgramContext;
use crate::error::Result;

pub trait Draw {
    fn draw(
        self: Box<Self>,
        register: &mut FramebufferContext,
        marker: &mut ShaderProgramContext,
    ) -> Result<()>;
}
