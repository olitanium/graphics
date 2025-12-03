use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Vector;
use graphics::{ActiveShaderProgram, Model};

use crate::EngResult;
use crate::modelling::cubic::geometry::CalculatedAnimation;
use crate::modelling::{CubicVertex, Skeleton};

pub trait EntityInner: Sized + Model
where
    Self::Vertex: CubicVertex,
{
    type BoundingBox<T>;

    fn make_bounding_box<T, F: FnMut(&[(Vector<3>, [Self::Vertex; 3])]) -> T>(
        &self,
        depth: usize,
        init: F,
    ) -> Option<Self::BoundingBox<T>>;

    fn skeleton(&self) -> &Skeleton;

    fn draw<'a, NEVER, const OUT: usize, D: FramebufferWithDepth<OUT>, L>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, Self, L, D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        calculated_animation: &CalculatedAnimation,
    ) -> EngResult<(), NEVER>;
}
