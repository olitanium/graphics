use std::fmt::Debug;

use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::{Matrix, Vector};
use graphics::shader_program::{ShaderProgram, ShaderProgramContext, Uniform};
use graphics::texture::FlatTexture;
use graphics::{ApiResult, Draw, DrawOut};

use crate::EngError;
use crate::entity::{AggregateEntity, AggregateModelIndex, EntityInner};
use crate::modelling::CubicVertex;
use crate::modelling::cubic::camera::CameraPose;
use crate::modelling::cubic::geometry::{Pose, SkeletalAnimation};
use crate::modelling::cubic::lighting::simple::ListLights;
use crate::modelling::cubic::{Camera, SkeletonBoneIndex};

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
    shader: &'a ShaderProgram<M, ListLights<MAX>, OUT, D::Tex>,
    framebuffer: &'a D,

    camera_pos: Vector<3>,
    camera_look_at: Matrix<4, 4>,

    lights: &'a ListLights<MAX>,
    opaque: Vec<(
        &'a AggregateEntity<T, M>,
        Vec<Option<(&'a SkeletalAnimation, f32)>>,
    )>,

    fix_anim: Option<(AggregateModelIndex, SkeletonBoneIndex, Pose)>,
    uniforms: Vec<(String, &'a dyn Uniform)>,
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
        shader: &'a ShaderProgram<M, ListLights<MAX>, OUT, FlatTexture>,
        framebuffer: &'a D,

        camera: &Camera<O>,
        hint: &O::Hint,
        lights: &'a ListLights<MAX>,
        opaque: Vec<(
            &'a AggregateEntity<T, M>,
            Vec<Option<(&'a SkeletalAnimation, f32)>>, // time
        )>,
        fix_anim: Option<(AggregateModelIndex, SkeletonBoneIndex, Pose)>,

        uniforms: Vec<(String, &'a dyn Uniform)>,
    ) -> Box<Self> {
        Box::new(Self {
            shader,
            framebuffer,
            camera_pos: camera.position(hint),
            camera_look_at: camera.look_at(hint),
            lights,
            opaque,
            uniforms,
            fix_anim,
        })
    }
}

impl<
    'a,
    T: Debug,
    M: EntityInner,
    NEVER,
    const MAX: usize,
    const OUT: usize,
    D: FramebufferWithDepth<OUT>,
> Draw<EngError<NEVER>> for Group<'a, M, T, MAX, OUT, D>
where
    M::Vertex: CubicVertex,
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

        for (model, animations) in self.opaque {
            model.draw(
                &mut active_shader,
                &mut active_framebuffer,
                animations,
                self.fix_anim.as_ref(),
            )?;
        }
        DrawOut::OK
    }
}
