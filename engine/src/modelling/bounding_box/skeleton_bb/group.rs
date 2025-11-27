use graphics::colour::ColourRGBA;
use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::{Matrix, Vector};
use graphics::texture::FlatTexture;
use graphics::vertex::Vertex;
use graphics::{ApiResult, Draw, DrawOut, ShaderProgramContext};

use crate::modelling::cubic::Camera;
use crate::modelling::cubic::camera::CameraPose;
use crate::modelling::cubic::geometry::{CalculatedAnimation, Pose, SkeletalAnimation};
use crate::modelling::{CubicVertex, Skeleton, SkeletonBb, Volume};
use crate::opengl_shaders::BoundingBoxShaderProgram;
use crate::{EngError, EngResult};

pub struct Group<
    'a,
    const OUT: usize,
    const DIMS: usize,
    D: FramebufferWithDepth<OUT, Tex = FlatTexture>,
    V: CubicVertex,
    T,
    F: Fn(&T) -> ColourRGBA,
> where
    Vector<DIMS>: Volume<DIMS>,
    [(); Vector::<DIMS>::ELEMENT_COUNT]:,
{
    framebuffer: &'a D,
    bounding_box: &'a SkeletonBb<V, T>,
    calculated_animation: CalculatedAnimation,
    look_at: Matrix<4, 4>,
    level: usize,
    colour: F,
}

impl<'a, D: FramebufferWithDepth<1, Tex = FlatTexture>, V: CubicVertex, T, F: Fn(&T) -> ColourRGBA>
    Group<'a, 1, 3, D, V, T, F>
{
    pub fn new<P: CameraPose, NEVER>(
        framebuffer: &'a D,
        bounding_box: &'a SkeletonBb<V, T>,
        // skeleton: &'a Skeleton,
        pose: Pose,
        animation: Option<(&'a SkeletalAnimation, f32)>,
        camera: &Camera<P>,
        hint: &P::Hint,
        level: usize,
        colour: F,
    ) -> EngResult<Box<Self>, NEVER> {
        let look_at = camera.look_at(hint);
        let calculated_animation =
            CalculatedAnimation::builder(pose, &bounding_box.skeleton, animation)?
                .build()
                .swap_remove(0); //Take 0 because there are no aggregate groups
        Ok(Box::new(Self {
            framebuffer,
            bounding_box,
            calculated_animation,
            look_at,
            level,
            colour,
        }))
    }
}

impl<
    'a,
    NEVER,
    D: FramebufferWithDepth<1, Tex = FlatTexture>,
    V: CubicVertex,
    T,
    F: Fn(&T) -> ColourRGBA,
> Draw<EngError<NEVER>> for Group<'a, 1, 3, D, V, T, F>
{
    fn draw(
        self: Box<Self>,
        register: &mut FramebufferContext,
        marker: &mut ShaderProgramContext,
    ) -> ApiResult<DrawOut, EngError<NEVER>> {
        let mut active_framebuffer = self.framebuffer.bind(register);
        let mut active_shader = Vector::shader_program().use_program(marker);

        active_shader.set_uniform("projtimesview", &self.look_at)?;

        self.bounding_box.draw(
            self.level,
            &mut active_shader,
            &mut active_framebuffer,
            self.calculated_animation,
            self.colour,
        )?;

        DrawOut::OK
    }
}
