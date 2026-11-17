use core::error::Error;
use core::iter;

use graphics::framebuffer::attachments::WithoutExtra;
use graphics::framebuffer::traits::{FramebufferWithDepth, FramebufferWithoutExtra};
use graphics::framebuffer::{Framebuffer, FramebufferContext};
use graphics::texture::{FlatTexture, Texture};
use graphics::{Draw, DrawOut, ShaderProgramContext};
use triangle_intersection::Vector;

use crate::deferred::{GBuffer, GBufferFramebufferQuad};
use crate::modelling::FramebufferQuad;
use crate::modelling::cubic::lighting::shadow::ShadowListLights;
use crate::modelling::cubic::{Camera, CameraPose};
use crate::opengl_shaders;

pub const SHADOW_DEFERRED_GROUP_MAX_LIGHTS: usize = 2;

pub struct ShadowDeferredCompleteGroup<'a, X, const OUT: usize>
where
    X: FramebufferWithoutExtra<OUT, Tex = FlatTexture>,
{
    framebuffer: &'a X,
    gbuffer: GBufferFramebufferQuad<'a>,
    ambient_occlusion: Option<&'a Framebuffer<1, WithoutExtra>>,
    lights: ShadowListLights<'a, SHADOW_DEFERRED_GROUP_MAX_LIGHTS>,
    camera_pos: Vector<3>,
}

impl<'a, X, const OUT: usize> ShadowDeferredCompleteGroup<'a, X, OUT>
where
    X: FramebufferWithoutExtra<OUT, Tex = FlatTexture>,
{
    pub fn new<P>(
        framebuffer: &'a X,
        complete_gbuffer: &'a GBuffer,
        ambient_occlusion: Option<&'a Framebuffer<1, WithoutExtra>>,
        lights: ShadowListLights<'a, SHADOW_DEFERRED_GROUP_MAX_LIGHTS>,
        camera: &Camera<P>,
        hint: &P::Hint,
    ) -> Box<Self>
    where
        P: CameraPose,
    {
        Box::new(Self {
            framebuffer,
            gbuffer: FramebufferQuad::new_from_framebuffer(complete_gbuffer),
            ambient_occlusion,
            lights,
            camera_pos: camera.position(hint),
        })
    }
}

impl<X> Draw for ShadowDeferredCompleteGroup<'_, X, 1>
where
    X: FramebufferWithDepth<1, Tex = FlatTexture>,
{
    fn draw_sized(
        self,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>>
    where
        Self: Sized,
    {
        let ao_ref = match self.ambient_occlusion {
            None => None, // FlatTexture::white_ref() as &dyn Texture,
            Some(x) => {
                let [ao] = x.get_all_raw();
                Some(ao.try_borrow()?)
            }
        };

        let ao = ao_ref.as_ref().map_or_else(
            || FlatTexture::white_ref() as &dyn Texture,
            |tex| &**tex as &dyn Texture,
        );

        let mut active_shader = opengl_shaders::deferred_shadow_process().use_program(sp_context);
        let mut active_framebuffer = self.framebuffer.bind(fb_context)?;

        self.lights.bind(&mut active_shader, self.camera_pos)?;

        active_shader.set_uniform(&c"camera_position", &self.camera_pos)?;

        active_shader.register_texture(iter::once((c"ambient_occlusion".to_owned(), ao)))?;

        self.gbuffer
            .draw(&mut active_shader, &mut active_framebuffer)?;

        drop(ao_ref);

        active_framebuffer.copy_depth_buffer::<4, _>(self.gbuffer.inner);

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

impl<X> Draw for ShadowDeferredCompleteGroup<'_, X, 3>
where
    X: FramebufferWithDepth<3, Tex = FlatTexture>,
{
    fn draw_sized(
        self,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>>
    where
        Self: Sized,
    {
        let ao_ref = match self.ambient_occlusion {
            None => None, // FlatTexture::white_ref() as &dyn Texture,
            Some(x) => {
                let [ao] = x.get_all_raw();
                Some(ao.try_borrow()?)
            }
        };

        let ao = ao_ref.as_ref().map_or_else(
            || FlatTexture::white_ref() as &dyn Texture,
            |tex| &**tex as &dyn Texture,
        );

        let mut active_shader =
            opengl_shaders::deferred_shadow_process_hdr().use_program(sp_context);
        let mut active_framebuffer = self.framebuffer.bind(fb_context)?;

        self.lights.bind(&mut active_shader, self.camera_pos)?;

        active_shader.register_texture(iter::once((c"ambient_occlusion".to_owned(), ao)))?;

        active_shader.set_uniform(&c"camera_position", &self.camera_pos)?;

        self.gbuffer
            .draw(&mut active_shader, &mut active_framebuffer)?;

        drop(ao_ref);

        active_framebuffer.copy_depth_buffer::<4, _>(self.gbuffer.inner);

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
