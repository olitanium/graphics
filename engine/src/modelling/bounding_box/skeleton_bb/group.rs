use graphics::colour::ColourRGBA;
use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Matrix;
use graphics::texture::FlatTexture;
use graphics::{Draw, DrawOut, ShaderProgramContext};

use crate::modelling::bounding_box::BoundingBoxShaderProgram;
use crate::modelling::cubic::Camera;
use crate::modelling::cubic::camera::CameraPose;
use crate::modelling::cubic::geometry::{
    CalculatedAnimation,
    Pose,
    SignaturesDoNotMatchError,
    SkeletalAnimation,
};
use crate::modelling::{BoundingBoxElement, CubicVertex, Skeleton, SkeletonBb};

#[derive(Debug)]
pub struct Group<
    'a,
    const OUT: usize,
    D: FramebufferWithDepth<OUT, Tex = FlatTexture>,
    V: CubicVertex,
    T,
    F: FnMut() -> ColourRGBA,
> {
    framebuffer: &'a D,
    bounding_box: &'a SkeletonBb<V, T>,
    calculated_animation: CalculatedAnimation,
    skeleton: &'a Skeleton,
    look_at: Matrix<4, 4>,
    level: usize,
    colour: F,
}

impl<'a, D: FramebufferWithDepth<1, Tex = FlatTexture>, V: CubicVertex, T, F: FnMut() -> ColourRGBA>
    Group<'a, 1, D, V, T, F>
{
    pub fn new<P: CameraPose>(
        framebuffer: &'a D,
        bounding_box: &'a SkeletonBb<V, T>,
        skeleton: &'a Skeleton,
        pose: Pose,
        animation: Option<(&'a SkeletalAnimation, f32)>,
        camera: &Camera<P>,
        hint: &P::Hint,
        level: usize,
        colour: F,
    ) -> Result<Box<Self>, SignaturesDoNotMatchError> {
        let look_at = camera.look_at(hint);
        let calculated_animation = CalculatedAnimation::new(pose, skeleton, animation)?;

        Ok(Box::new(Self {
            framebuffer,
            bounding_box,
            calculated_animation,
            skeleton,
            look_at,
            level,
            colour,
        }))
    }
}

impl<'a, D: FramebufferWithDepth<1, Tex = FlatTexture>, V: CubicVertex, T, F: FnMut() -> ColourRGBA>
    Draw for Group<'a, 1, D, V, T, F>
{
    fn draw_sized(
        self,
        register: &mut FramebufferContext,
        marker: &mut ShaderProgramContext,
    ) -> DrawOut {
        let mut active_framebuffer = self.framebuffer.bind(register);
        let mut active_shader = BoundingBoxElement::<V, 3, T>::shader_program().use_program(marker);

        active_shader
            .set_uniform("projtimesview", &self.look_at)
            .unwrap();

        self.bounding_box
            .draw(
                self.level,
                &mut active_shader,
                &mut active_framebuffer,
                self.skeleton,
                self.calculated_animation,
                self.colour,
            )
            .expect("calculated animation was created from given skeleton");

        DrawOut::OK
    }

    fn draw(
        self: Box<Self>,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> DrawOut {
        let inner = *self;
        inner.draw_sized(fb_context, sp_context)
    }
}
