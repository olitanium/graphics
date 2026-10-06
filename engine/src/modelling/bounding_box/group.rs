use core::error::Error;
use core::fmt::Debug;

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
#[must_use = "Group must be sent to the GPU for drawing"]
pub struct Group<
    'a,
    const OUT: usize,
    D: FramebufferWithDepth<OUT, Tex = FlatTexture>,
    V: ExtendedVertex,
    T,
    Bt,
    F: FnMut() -> ColourRGBA,
> where
    BoundingBoxElement<V, Bt>: BoundingBoxShaderProgram<DIMS = { V::LEN }>,
    T: Transform<Vector<{ V::LEN }>>,
    Vector<{ V::LEN }>: Volume + Vertex,
{
    framebuffer: &'a D,
    bounding_box: &'a BoundingBox<V, Bt>,
    transform: &'a T,
    look_at: Matrix<4, 4>,
    level: usize,
    colour: F,
}

impl<
    'a,
    D: FramebufferWithDepth<1, Tex = FlatTexture>,
    V: ExtendedVertex,
    Bt,
    T,
    F: FnMut() -> ColourRGBA,
> Group<'a, 1, D, V, T, Bt, F>
where
    BoundingBoxElement<V, Bt>: BoundingBoxShaderProgram<DIMS = { V::LEN }>,
    T: Transform<Vector<{ V::LEN }>>,
    Vector<{ V::LEN }>: Volume,
{
    pub fn new<P>(
        framebuffer: &'a D,
        bounding_box: &'a BoundingBox<V, Bt>,
        transform: &'a T,
        camera: &Camera<P>,
        hint: &P::Hint,
        level: usize,
        colour: F,
    ) -> Box<Self>
    where
        P: CameraPose,
    {
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

impl<D, V, T, Bt, F> Draw for Group<'_, 1, D, V, T, Bt, F>
where
    V: ExtendedVertex,
    F: FnMut() -> ColourRGBA,
    D: FramebufferWithDepth<1, Tex = FlatTexture>,
    Vector<{ V::LEN }>: Volume,
    T: Transform<Vector<{ V::LEN }>> + Clone,
    BoundingBoxElement<V, Bt>: BoundingBoxShaderProgram<DIMS = { V::LEN }>,
{
    fn draw_sized(
        self,
        fb_context: &mut FramebufferContext,
        sp_context: &mut ShaderProgramContext,
    ) -> Result<DrawOut, Box<dyn Error>> {
        let mut active_framebuffer = self.framebuffer.bind(fb_context)?;
        let mut active_shader =
            BoundingBoxElement::<V, Bt>::shader_program().use_program(sp_context);

        active_shader.set_uniform(&c"projtimesview", &self.look_at)?;

        self.bounding_box.draw(
            self.level,
            self.transform,
            &mut active_shader,
            &mut active_framebuffer,
            self.colour,
        )?;

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
