use crate::framebuffer::FramebufferContext;
use crate::shader_program::ShaderProgramContext;

pub trait Draw {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        shader_marker: &mut ShaderProgramContext,
    );
}

impl<T: Draw> Draw for Vec<T> {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        shader_marker: &mut ShaderProgramContext,
    ) {
        for group in *self {
            Box::new(group).draw(fb_context, shader_marker);
        }
    }
}

impl Draw for Vec<Box<dyn Draw>> {
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        shader_marker: &mut ShaderProgramContext,
    ) {
        for group in *self {
            group.draw(fb_context, shader_marker);
        }
    }
}
