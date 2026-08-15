use std::fmt::Debug;

use geometry::Transform;
use graphics::colour::ColourRGBA;
use graphics::framebuffer::FramebufferContext;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::{Matrix, Vector};
use graphics::texture::FlatTexture;
use graphics::vertex::Vertex;
use graphics::{Draw, DrawOut, ShaderProgramContext};

use crate::modelling::bounding_box::BoundingBoxShaderProgram;
use crate::modelling::cubic::{Camera, CameraPose};
use crate::modelling::{BoundingBox, BoundingBoxElement, ExtendedVertex, Volume};

#[derive(Debug)]
pub struct Group<
    'a,
    const OUT: usize,
    const DIMS: usize,
    D: FramebufferWithDepth<OUT, Tex = FlatTexture>,
    V: ExtendedVertex<DIMS>,
    T: Transform<Vector<DIMS>>,
    Bt,
    F: FnMut() -> ColourRGBA,
> where
    BoundingBoxElement<V, DIMS, Bt>: BoundingBoxShaderProgram<DIMS>,
    T: Transform<Vector<DIMS>>,
    Vector<DIMS>: Volume<DIMS>,
    [(); <Vector<DIMS> as Vertex>::ELEMENT_COUNT]:,
{
    framebuffer: &'a D,
    bounding_box: &'a BoundingBox<V, DIMS, Bt>,
    transform: T,
    look_at: Matrix<4, 4>,
    level: usize,
    colour: F,
}

impl<
    'a,
    const DIMS: usize,
    D: FramebufferWithDepth<1, Tex = FlatTexture>,
    V: ExtendedVertex<DIMS>,
    Bt,
    T,
    F: FnMut() -> ColourRGBA,
> Group<'a, 1, DIMS, D, V, T, Bt, F>
where
    BoundingBoxElement<V, DIMS, Bt>: BoundingBoxShaderProgram<DIMS>,
    T: Transform<Vector<DIMS>>,
    Vector<DIMS>: Volume<DIMS>,
    [(); <Vector<DIMS> as Vertex>::ELEMENT_COUNT]:,
{
    pub fn new<P: CameraPose>(
        framebuffer: &'a D,
        bounding_box: &'a BoundingBox<V, DIMS, Bt>,
        transform: T,
        camera: &Camera<P>,
        hint: &P::Hint,
        level: usize,
        colour: F,
    ) -> Box<Self> {
        let look_at = camera.look_at(hint);
        Box::new(Self {
            framebuffer,
            bounding_box,
            transform,
            look_at,
            level,
            colour,
        })
    }
}

impl<
    'a,
    const DIMS: usize,
    D: FramebufferWithDepth<1, Tex = FlatTexture>,
    V: ExtendedVertex<DIMS>,
    T,
    Bt,
    F: FnMut() -> ColourRGBA,
> Draw for Group<'a, 1, DIMS, D, V, T, Bt, F>
where
    Vector<DIMS>: Volume<DIMS>,
    T: Transform<Vector<DIMS>>,
    BoundingBoxElement<V, DIMS, Bt>: BoundingBoxShaderProgram<DIMS>,
    [(); <Vector<DIMS> as Vertex>::ELEMENT_COUNT]:,
{
    fn draw_sized(
        self,
        register: &mut FramebufferContext,
        marker: &mut ShaderProgramContext,
    ) -> DrawOut {
        let mut active_framebuffer = self.framebuffer.bind(register);
        let mut active_shader =
            BoundingBoxElement::<V, DIMS, Bt>::shader_program().use_program(marker);

        active_shader
            .set_uniform("projtimesview", &self.look_at)
            .unwrap();

        self.bounding_box
            .draw(
                self.level,
                self.transform,
                &mut active_shader,
                &mut active_framebuffer,
                self.colour,
            )
            .unwrap();

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
