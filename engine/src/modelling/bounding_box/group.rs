use graphics::colour::ColourRGBA;
use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::{Matrix, Vector};
use graphics::texture::FlatTexture;
use graphics::vertex::Vertex;
use graphics::{ApiResult, Draw, DrawOut, ShaderProgramContext};

use crate::EngError;
use crate::modelling::cubic::Camera;
use crate::modelling::cubic::camera::CameraPose;
use crate::modelling::cubic::geometry::Pose;
use crate::modelling::{BoundingBox, CubicVertex, ExtendedVertex, Volume};
use crate::opengl_shaders::BoundingBoxShaderProgram;
pub struct Group<
    'a,
    const OUT: usize,
    const DIMS: usize,
    D: FramebufferWithDepth<OUT, Tex = FlatTexture>,
    V: ExtendedVertex<DIMS>,
    T,
    F: Fn(&T) -> ColourRGBA,
> where
    Vector<DIMS>: Volume<DIMS>,
    [(); Vector::<DIMS>::ELEMENT_COUNT]:,
{
    framebuffer: &'a D,
    bounding_box: &'a BoundingBox<V, DIMS, T>,
    pose: Pose,
    look_at: Matrix<4, 4>,
    level: usize,
    colour: F,
}

impl<
    'a,
    D: FramebufferWithDepth<1, Tex = FlatTexture>,
    V: ExtendedVertex<3>,
    T,
    F: Fn(&T) -> ColourRGBA,
> Group<'a, 1, 3, D, V, T, F>
{
    pub fn new<P: CameraPose>(
        framebuffer: &'a D,
        bounding_box: &'a BoundingBox<V, 3, T>,
        pose: Pose,
        camera: &Camera<P>,
        hint: &P::Hint,
        level: usize,
        colour: F,
    ) -> Box<Self> {
        let look_at = camera.look_at(hint);
        Box::new(Self {
            framebuffer,
            bounding_box,
            pose,
            look_at,
            level,
            colour,
        })
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
            self.pose,
            &mut active_shader,
            &mut active_framebuffer,
            self.colour,
        )?;

        DrawOut::OK
    }
}
