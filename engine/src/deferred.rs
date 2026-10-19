use core::error::Error;

use graphics::framebuffer::attachments::WithStencil;
use graphics::framebuffer::traits::FramebufferWithoutExtra;
use graphics::framebuffer::{Framebuffer, FramebufferContext};
use graphics::texture::FlatTexture;
use graphics::{Draw, DrawOut, ShaderProgramContext};

use crate::modelling::FramebufferQuad;
use crate::modelling::cubic::lighting::simple::ListLights;
use crate::opengl_shaders;

pub const GBUFFER_ELEMENTS: usize = 3;

pub type GBuffer = Framebuffer<GBUFFER_ELEMENTS, WithStencil>;

pub const SIMPLE_DEFERRED_GROUP_MAX_LIGHTS: usize = 2;

pub struct SimpleDeferredCompleteGroup<'a, X>
where
    X: FramebufferWithoutExtra<1, Tex = FlatTexture>,
{
    framebuffer: &'a X,
    gbuffer: FramebufferQuad<'a, GBUFFER_ELEMENTS, WithStencil>,
    lights: ListLights<'a, SIMPLE_DEFERRED_GROUP_MAX_LIGHTS>,
}

impl<'a, X> SimpleDeferredCompleteGroup<'a, X>
where
    X: FramebufferWithoutExtra<1, Tex = FlatTexture>,
{
    pub fn new(
        framebuffer: &'a X,
        complete_gbuffer: &'a GBuffer,
        lights: ListLights<'a, SIMPLE_DEFERRED_GROUP_MAX_LIGHTS>,
    ) -> Box<Self> {
        Box::new(Self {
            framebuffer,
            gbuffer: FramebufferQuad::new_from_framebuffer(complete_gbuffer),
            lights,
        })
    }
}

impl<X> Draw for SimpleDeferredCompleteGroup<'_, X>
where
    X: FramebufferWithoutExtra<1, Tex = FlatTexture>,
{
    fn draw_sized(
        self,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut active_shader = opengl_shaders::deferred_simple_process().use_program(sp_context);
        let mut active_framebuffer = self.framebuffer.bind(fb_context)?;

        self.lights.bind(&mut active_shader)?;

        self.gbuffer
            .draw(&mut active_shader, &mut active_framebuffer)?;

        DrawOut::OK
    }

    fn draw(
        self: Box<Self>,
        fb_context: &mut graphics::framebuffer::FramebufferContext,
        sp_context: &mut graphics::ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>> {
        let inner = *self;
        inner.draw_sized(fb_context, sp_context)
    }
}
