use alloc::ffi::CString;
use core::error::Error;

use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::{Matrix, Vector};
use graphics::shader_program::{ShaderProgram, ShaderProgramContext, Uniform};
use graphics::texture::FlatTexture;
use graphics::{Draw, DrawOut};

use crate::modelling::SkeletonCubic;
use crate::modelling::cubic::camera::CameraPose;
use crate::modelling::cubic::lighting::simple::ListLights;
use crate::modelling::cubic::{CalculatedAnimation, Camera, SignaturesDoNotMatchError};

#[derive(Debug)]
pub struct Group<'a, const MAX: usize, const OUT: usize, D: FramebufferWithDepth<OUT>> {
    shader: &'a ShaderProgram<SkeletonCubic, ListLights<'a, MAX>, OUT, D::Tex>,
    framebuffer: &'a D,

    camera_pos: Vector<3>,
    camera_look_at: Matrix<4, 4>,

    lights: ListLights<'a, MAX>,
    opaque: Vec<(&'a SkeletonCubic, &'a CalculatedAnimation)>,

    uniforms: Vec<(CString, &'a (dyn Uniform + 'a))>,
}

impl<'a, const MAX: usize, const OUT: usize, D: FramebufferWithDepth<OUT, Tex = FlatTexture>>
    Group<'a, MAX, OUT, D>
{
    #[must_use = "Group must be sent to the GPU for drawing"]
    pub fn new<O>(
        shader: &'a ShaderProgram<SkeletonCubic, ListLights<'a, MAX>, OUT, FlatTexture>,
        framebuffer: &'a D,

        camera: &Camera<O>,
        hint: &O::Hint,
        lights: ListLights<'a, MAX>,
        opaque: Vec<(&'a SkeletonCubic, &'a CalculatedAnimation)>,

        uniforms: Vec<(CString, &'a (dyn Uniform + 'a))>,
    ) -> Result<Box<Self>, SignaturesDoNotMatchError>
    where
        O: CameraPose,
    {
        Ok(Box::new(Self {
            shader,
            framebuffer,
            camera_pos: camera.position(hint),
            camera_look_at: camera.look_at(hint),
            lights,
            opaque,
            uniforms,
        }))
    }
}

impl<const MAX: usize, const OUT: usize, D: FramebufferWithDepth<OUT>> Draw
    for Group<'_, MAX, OUT, D>
{
    fn draw_sized(
        self,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>> {
        let mut active_shader = self.shader.use_program(sp_context);
        let mut active_framebuffer = self.framebuffer.bind(fb_context)?;

        self.lights.bind(&mut active_shader)?;

        active_shader.set_uniform(&c"projtimesview", &self.camera_look_at)?;
        active_shader.set_uniform(&c"camera_position", &self.camera_pos.homogeneous())?;

        active_shader.set_uniform_dyn(self.uniforms)?;

        let err: Vec<_> = self
            .opaque
            .into_iter()
            .filter_map(|(model, calculated_animation)| {
                model
                    .draw(
                        &mut active_shader,
                        &mut active_framebuffer,
                        calculated_animation,
                    )
                    .err()
            })
            .collect();

        err.is_empty().ok_or_else(|| format!("{err:?}"))?;

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
