use graphics::Draw;
use graphics::error::Result;
use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::{FramebufferInternals, FramebufferWithoutExtra};
use graphics::shader_program::ShaderProgramContext;
use graphics::texture::FlatTexture;

use super::Bloom;
use crate::modelling::Quad;
use crate::opengl_shaders;

#[derive(Debug)]
pub struct Group<'a, X: FramebufferWithoutExtra<1, Tex = FlatTexture>> {
    bloom: &'a Bloom,
    output_fb: &'a X,
}

impl<'a, X: FramebufferWithoutExtra<1, Tex = FlatTexture>> Group<'a, X> {
    pub fn new(output_fb: &'a X, bloom: &'a Bloom) -> Box<Self> {
        Box::new(Self { bloom, output_fb })
    }
}

impl<'a, X: FramebufferWithoutExtra<1, Tex = FlatTexture>> Draw for Group<'a, X> {
    fn draw(
        self: Box<Self>,
        register: &mut FramebufferContext,
        marker: &mut ShaderProgramContext,
    ) -> Result<()> {
        let Group { bloom, output_fb } = *self;

        let mut active_framebuffer_x = bloom.framebuffer_x.bind(register);
        let active_blur_x = opengl_shaders::bloom_x().use_program(marker);
        bloom
            .to_blur
            .draw(active_blur_x, &mut active_framebuffer_x)?;

        let mut active_output_fb = output_fb.bind(register);
        let active_blur_y = opengl_shaders::bloom_y().use_program(marker);

        Quad::screen(bloom.framebuffer_x.get_all_colour())
            .draw(active_blur_y, &mut active_output_fb)?;

        Ok(())
    }
}
