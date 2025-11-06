use crate::ApiResult;
use crate::framebuffer::FramebufferContext;
use crate::shader_program::ShaderProgramContext;

pub trait Draw<E> {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        shader_marker: &mut ShaderProgramContext,
    ) -> ApiResult<(), E>;
}

impl<T: Draw<E>, E> Draw<E> for Vec<T> {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        shader_marker: &mut ShaderProgramContext,
    ) -> ApiResult<(), E> {
        for group in *self {
            Box::new(group).draw(fb_context, shader_marker)?;
        }

        Ok(())
    }
}
// impl<E: Error> Draw<E> for Vec<Box<dyn Draw<E>>> {
// fn draw(
// self: Box<Self>,
// fb_context: &mut FramebufferContext,
// shader_marker: &mut ShaderProgramContext,
// ) {
// for group in *self {
// group.draw(fb_context, shader_marker);
// }
// }
// }
