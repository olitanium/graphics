use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::{Matrix, Vector};
use graphics::shader_program::{ShaderProgram, ShaderProgramContext, Uniform};
use graphics::texture::FlatTexture;
use graphics::{ApiResult, Draw, DrawOut};

use crate::modelling::cubic::Camera;
use crate::modelling::cubic::camera::CameraPose;
use crate::modelling::cubic::geometry::{
    CalculatedAnimation,
    Pose,
    SkeletalAnimation,
    process_vec_noaggregate,
};
use crate::modelling::cubic::lighting::simple::ListLights;
use crate::modelling::SimpleCubic;
use crate::{EngError, EngResult};

#[derive(Debug)]
pub struct Group<'a, const MAX: usize, const OUT: usize, D: FramebufferWithDepth<OUT>> {
    shader: &'a ShaderProgram<SimpleCubic, ListLights<MAX>, OUT, D::Tex>,
    framebuffer: &'a D,

    camera_pos: Vector<3>,
    camera_look_at: Matrix<4, 4>,

    lights: &'a ListLights<MAX>,
    opaque: Vec<(&'a SimpleCubic, CalculatedAnimation)>,

    uniforms: Vec<(String, &'a dyn Uniform)>,
}

impl<'a, const MAX: usize, const OUT: usize, D: FramebufferWithDepth<OUT, Tex = FlatTexture>>
    Group<'a, MAX, OUT, D>
{
    pub fn new<O: CameraPose, NEVER>(
        shader: &'a ShaderProgram<SimpleCubic, ListLights<MAX>, OUT, FlatTexture>,
        framebuffer: &'a D,

        camera: &Camera<O>,
        hint: &O::Hint,
        lights: &'a ListLights<MAX>,
        opaque: Vec<(
            &'a SimpleCubic,
            Pose,
            Option<(&'a SkeletalAnimation, f32)>, // animation
        )>,

        uniforms: Vec<(String, &'a dyn Uniform)>,
    ) -> EngResult<Box<Self>, NEVER> {
        let opaque = process_vec_noaggregate(opaque)?;
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

impl<'a, NEVER, const MAX: usize, const OUT: usize, D: FramebufferWithDepth<OUT>>
    Draw<EngError<NEVER>> for Group<'a, MAX, OUT, D>
{
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> ApiResult<DrawOut, EngError<NEVER>> {
        let mut active_shader = self.shader.use_program(sp_context);
        let mut active_framebuffer = self.framebuffer.bind(fb_context);

        self.lights.bind(&mut active_shader)?;

        active_shader.set_uniform("projtimesview".to_string(), &self.camera_look_at)?;
        active_shader.set_uniform(
            "camera_position".to_string(),
            &self.camera_pos.homogeneous(),
        )?;

        active_shader.set_uniform_dyn(self.uniforms)?;

        for (model, calculated_animation) in self.opaque {
            model.draw(
                &mut active_shader,
                &mut active_framebuffer,
                &calculated_animation,
            )?;
        }
        DrawOut::OK
    }
}
