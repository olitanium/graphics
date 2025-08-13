use crate::error::ApiResult;
use crate::framebuffer::FramebufferContext;
use crate::shader_program::ShaderProgramContext;

pub trait Draw<E> {
    fn draw(
        self: Box<Self>,
        register: &mut FramebufferContext,
        marker: &mut ShaderProgramContext,
    ) -> ApiResult<(), E>;
}
