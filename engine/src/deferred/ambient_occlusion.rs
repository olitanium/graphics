mod utils;

use core::error;
use core::sync::atomic::{AtomicU32, Ordering};

use graphics::framebuffer::attachments::WithoutExtra;
use graphics::framebuffer::traits::FramebufferWithoutExtra as _;
use graphics::framebuffer::{Framebuffer, FramebufferContext};
use graphics::linear_algebra::Matrix;
use graphics::texture::Texture;
use graphics::{Draw, DrawOut, ShaderProgramContext};
pub use utils::{RANDOM_VEC_KERNEL, SAMPLE_VECS};

use crate::deferred::ambient_occlusion::utils::{KERNEL_SIZE, get_many_sample_vecs};
use crate::deferred::{GBuffer, GBufferFramebufferQuad};
use crate::modelling::FramebufferQuad;
use crate::modelling::cubic::{Camera, CameraPose};
use crate::opengl_shaders;

pub struct MakeAmbientOcclusionGroup<'a> {
    gbuffer: GBufferFramebufferQuad<'a>,
    ambient_occlusion_buffer: &'a Framebuffer<1, WithoutExtra>,
    look_at: Matrix<4, 4>,
    sample_vecs_len: u32,
    toggle: i32,
}

impl<'a> MakeAmbientOcclusionGroup<'a> {
    pub fn new<P>(
        gbuffer: &'a GBuffer,
        ambient_occlusion_buffer: &'a Framebuffer<1, WithoutExtra>,
        sample_vecs_len: u32,
        camera: &Camera<P>,
        hint: &P::Hint,
        toggle: i32,
    ) -> Box<Self>
    where
        P: CameraPose,
    {
        let look_at = camera.look_at(hint);
        let gbuffer = FramebufferQuad::new_from_framebuffer(gbuffer);
        Box::new(Self {
            gbuffer,
            ambient_occlusion_buffer,
            look_at,
            sample_vecs_len,
            toggle,
        })
    }
}

impl Draw for MakeAmbientOcclusionGroup<'_> {
    fn draw_sized(
        self,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn error::Error>>
    where
        Self: Sized,
    {
        static FRAMECOUNTER: AtomicU32 = AtomicU32::new(0);
        let mut active_sp = opengl_shaders::ambient_occlusion().use_program(sp_context);
        let mut active_fb = self.ambient_occlusion_buffer.bind(fb_context)?;

        let vecs: [_; 128] = *SAMPLE_VECS;
        active_sp.set_uniform(&c"look_at", &self.look_at)?;
        active_sp.set_uniform(&c"sample_vecs", vecs.as_slice())?;
        active_sp.set_uniform(
            &c"sample_vecs_len",
            &u32::min(self.sample_vecs_len, vecs.len() as u32),
        )?;

        active_sp.set_uniform(&c"kernel_size", &KERNEL_SIZE)?;

        active_sp.set_uniform(&c"frames", &FRAMECOUNTER.fetch_add(1, Ordering::Relaxed))?;

        active_sp.register_texture([
            (c"random_tangent", &*RANDOM_VEC_KERNEL as &dyn Texture),
            (c"depth_buffer", self.gbuffer.get_depth()),
        ])?;

        active_sp.set_uniform(&c"toggle_clip", &self.toggle)?;

        self.gbuffer.draw(&mut active_sp, &mut active_fb)?;

        DrawOut::OK
    }

    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn error::Error>> {
        self.draw_sized(fb_context, sp_context)
    }
}
