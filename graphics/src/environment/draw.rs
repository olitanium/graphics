use crate::error::Result;
use crate::framebuffer::FramebufferContext;
use crate::shader_program::ShaderProgramContext;

pub trait Draw {
    fn draw(
        self: Box<Self>,
        register: &mut FramebufferContext,
        marker: &mut ShaderProgramContext,
    ) -> Result<()>;
}
