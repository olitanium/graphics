use graphics::colour::ColourRGBA;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::framebuffer::{self, FramebufferContext};
use graphics::linear_algebra::{Matrix, Vector};
use graphics::texture::{FlatTexture, Texture};
use graphics::vertex::Vertex;
use graphics::{Draw, ShaderProgram, ShaderProgramContext};

use crate::EngResult;
use crate::modelling::cubic::Camera;
use crate::modelling::cubic::geometry::CameraPose;
use crate::modelling::{BoundingBox, ExtendedVertex, Skeleton, Volume, bounding_box};
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
    skeleton: &'a Skeleton,
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
        skeleton: &'a Skeleton,
        camera: &Camera<P>,
        hint: &P::Hint,
        level: usize,
        colour: F,
    ) -> Box<Self> {
        let look_at = camera.look_at(hint);
        Box::new(Self {
            framebuffer,
            bounding_box,
            skeleton,
            look_at,
            level,
            colour,
        })
    }
}

impl<
    'a,
    D: FramebufferWithDepth<1, Tex = FlatTexture>,
    V: ExtendedVertex<3>,
    T,
    F: Fn(&T) -> ColourRGBA,
> Draw for Group<'a, 1, 3, D, V, T, F>
{
    fn draw(self: Box<Self>, register: &mut FramebufferContext, marker: &mut ShaderProgramContext) {
        let _: EngResult<(), !> = try {
            let mut active_framebuffer = self.framebuffer.bind(register);
            let mut active_shader = Vector::shader_program().use_program(marker);

            // only valid for DIMS = 3
            active_shader.set_uniform("projtimesview", &self.look_at);

            self.bounding_box.draw(
                self.level,
                self.skeleton,
                &mut active_shader,
                &mut active_framebuffer,
                self.colour,
            )?
        };
    }
}
