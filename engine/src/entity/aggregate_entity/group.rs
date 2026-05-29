use std::fmt::Debug;

use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::{Matrix, Vector};
use graphics::shader_program::{ShaderProgram, ShaderProgramContext, Uniform};
use graphics::texture::FlatTexture;
use graphics::{Draw, DrawOut};

use crate::entity::{AggregateEntity, CalculateAnimationError, EntityInner};
use crate::modelling::CubicVertex;
use crate::modelling::cubic::Camera;
use crate::modelling::cubic::camera::CameraPose;
use crate::modelling::cubic::geometry::AggregateCalcAnim;
use crate::modelling::cubic::lighting::simple::ListLights;

pub struct Group<
    'a,
    M: EntityInner,
    T: Debug,
    const MAX: usize,
    const OUT: usize,
    D: FramebufferWithDepth<OUT>,
> where
    M::Vertex: CubicVertex,
{
    shader: &'a ShaderProgram<M, ListLights<'a, MAX>, OUT, D::Tex>,
    framebuffer: &'a D,

    camera_pos: Vector<3>,
    camera_look_at: Matrix<4, 4>,

    lights: ListLights<'a, MAX>,
    opaque: Vec<(&'a AggregateEntity<T, M>, AggregateCalcAnim)>,
    uniforms: Vec<(String, &'a dyn Uniform)>,
}

impl<
    'a,
    M: EntityInner,
    T: Debug,
    const MAX: usize,
    const OUT: usize,
    D: FramebufferWithDepth<OUT, Tex = FlatTexture>,
> Debug for Group<'a, M, T, MAX, OUT, D>
where
    M::Vertex: CubicVertex,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(Group)).finish_non_exhaustive()
    }
}

impl<
    'a,
    M: EntityInner,
    T: Debug,
    const MAX: usize,
    const OUT: usize,
    D: FramebufferWithDepth<OUT, Tex = FlatTexture>,
> Group<'a, M, T, MAX, OUT, D>
where
    M::Vertex: CubicVertex,
{
    pub fn new<O: CameraPose>(
        shader: &'a ShaderProgram<M, ListLights<'a, MAX>, OUT, FlatTexture>,
        framebuffer: &'a D,

        camera: &Camera<O>,
        hint: &O::Hint,
        lights: ListLights<'a, MAX>,
        opaque: Vec<(
            &'a AggregateEntity<T, M>,
            // HashMap<AggregateModelIndex, (&'a SkeletalAnimation, f32)>, // time
            // Option<(AggregateModelIndex, SkeletonBoneIndex, Pose)>,     // fix_anim
            AggregateCalcAnim,
        )>,

        uniforms: Vec<(String, &'a dyn Uniform)>,
    ) -> Result<Box<Self>, CalculateAnimationError<'a>> {
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

impl<'a, T: Debug, M: EntityInner, const MAX: usize, const OUT: usize, D: FramebufferWithDepth<OUT>>
    Draw for Group<'a, M, T, MAX, OUT, D>
where
    M::Vertex: CubicVertex,
{
    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> DrawOut {
        let mut active_shader = self.shader.use_program(sp_context);
        let mut active_framebuffer = self.framebuffer.bind(fb_context);

        self.lights.bind(&mut active_shader).unwrap();

        active_shader
            .set_uniform("projtimesview".to_string(), &self.camera_look_at)
            .unwrap();
        active_shader
            .set_uniform(
                "camera_position".to_string(),
                &self.camera_pos.homogeneous(),
            )
            .unwrap();

        active_shader.set_uniform_dyn(self.uniforms).unwrap();

        for (model, animations) in self.opaque {
            model
                .draw(&mut active_shader, &mut active_framebuffer, animations)
                .unwrap();
        }
        DrawOut::OK
    }
}
