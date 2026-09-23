use graphics::framebuffer::traits::{AttachmentWithoutExtra, FramebufferWithoutExtra};
use graphics::framebuffer::{Framebuffer, FramebufferContext};
use graphics::shader_program::ShaderProgramContext;
use graphics::texture::FlatTexture;
use graphics::{Draw, DrawOut};

use super::Bloom;
use crate::modelling::FramebufferQuad;
use crate::opengl_shaders;

#[derive(Debug)]
pub struct Group<
    'a,
    X: FramebufferWithoutExtra<1, Tex = FlatTexture>,
    Y: AttachmentWithoutExtra<Tex = FlatTexture>,
> {
    bloom: &'a Bloom,
    input_fb: &'a Framebuffer<3, Y>,
    output_fb: &'a X,
}

impl<
    'a,
    X: FramebufferWithoutExtra<1, Tex = FlatTexture>,
    Y: AttachmentWithoutExtra<Tex = FlatTexture>,
> Group<'a, X, Y>
{
    pub fn new(input_fb: &'a Framebuffer<3, Y>, output_fb: &'a X, bloom: &'a Bloom) -> Box<Self> {
        Box::new(Self {
            bloom,
            input_fb,
            output_fb,
        })
    }
}

impl<X: FramebufferWithoutExtra<1, Tex = FlatTexture>, Y: AttachmentWithoutExtra<Tex = FlatTexture>>
    Draw for Group<'_, X, Y>
{
    fn draw_sized(
        self,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> DrawOut {
        let mut active_framebuffer_x = self.bloom.framebuffer_x.bind(fb_context);
        let mut active_blur_x = opengl_shaders::bloom_x().use_program(sp_context);

        let input_quad = FramebufferQuad::new_from_framebuffer(self.input_fb);

        input_quad
            .draw(&mut active_blur_x, &mut active_framebuffer_x)
            .unwrap();

        drop(active_framebuffer_x);

        let mut active_output_fb = self.output_fb.bind(fb_context);
        let mut active_blur_y = opengl_shaders::bloom_y().use_program(sp_context);

        let blur_x_quad = FramebufferQuad::new_from_framebuffer(&self.bloom.framebuffer_x);

        blur_x_quad
            .draw(&mut active_blur_y, &mut active_output_fb)
            .unwrap();

        DrawOut::OK
    }

    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> DrawOut {
        let inner = *self;
        inner.draw_sized(fb_context, sp_context)
    }
}
