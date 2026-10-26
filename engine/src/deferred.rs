use core::error::Error;

use graphics::framebuffer::attachments::WithStencil;
use graphics::framebuffer::traits::FramebufferWithoutExtra;
use graphics::framebuffer::{Framebuffer, FramebufferContext};
use graphics::texture::FlatTexture;
use graphics::{Draw, DrawOut, ShaderProgramContext};
use triangle_intersection::Vector;

use crate::modelling::FramebufferQuad;
use crate::modelling::cubic::lighting::simple::ListLights;
use crate::modelling::cubic::{Camera, CameraPose};
use crate::opengl_shaders;

pub const GBUFFER_ELEMENTS: usize = 4;

pub type GBuffer = Framebuffer<GBUFFER_ELEMENTS, WithStencil>;

pub const SIMPLE_DEFERRED_GROUP_MAX_LIGHTS: usize = 2;

pub struct SimpleDeferredCompleteGroup<'a, X, const OUT: usize>
where
    X: FramebufferWithoutExtra<OUT, Tex = FlatTexture>,
{
    framebuffer: &'a X,
    gbuffer: FramebufferQuad<'a, GBUFFER_ELEMENTS, WithStencil>,
    lights: ListLights<'a, SIMPLE_DEFERRED_GROUP_MAX_LIGHTS>,
    camera_pos: Vector<3>,
}

impl<'a, X, const OUT: usize> SimpleDeferredCompleteGroup<'a, X, OUT>
where
    X: FramebufferWithoutExtra<OUT, Tex = FlatTexture>,
{
    pub fn new<P>(
        framebuffer: &'a X,
        complete_gbuffer: &'a GBuffer,
        lights: ListLights<'a, SIMPLE_DEFERRED_GROUP_MAX_LIGHTS>,
        camera: &Camera<P>,
        hint: &P::Hint,
    ) -> Box<Self>
    where
        P: CameraPose,
    {
        Box::new(Self {
            framebuffer,
            gbuffer: FramebufferQuad::new_from_framebuffer(complete_gbuffer),
            lights,
            camera_pos: camera.position(hint),
        })
    }
}

impl<X> Draw for SimpleDeferredCompleteGroup<'_, X, 1>
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

        active_shader.set_uniform(&c"camera_position", &self.camera_pos)?;

        self.gbuffer
            .draw(&mut active_shader, &mut active_framebuffer)?;

        DrawOut::OK
    }

    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>> {
        let inner = *self;
        inner.draw_sized(fb_context, sp_context)
    }
}

impl<X> Draw for SimpleDeferredCompleteGroup<'_, X, 3>
where
    X: FramebufferWithoutExtra<3, Tex = FlatTexture>,
{
    fn draw_sized(
        self,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>>
    where
        Self: Sized,
    {
        let mut active_shader =
            opengl_shaders::deferred_simple_process_hdr().use_program(sp_context);
        let mut active_framebuffer = self.framebuffer.bind(fb_context)?;

        self.lights.bind(&mut active_shader)?;

        active_shader.set_uniform(&c"camera_position", &self.camera_pos)?;

        self.gbuffer
            .draw(&mut active_shader, &mut active_framebuffer)?;

        DrawOut::OK
    }

    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>> {
        let inner = *self;
        inner.draw_sized(fb_context, sp_context)
    }
}
