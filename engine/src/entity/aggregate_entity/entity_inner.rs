use core::error::Error;

use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Vector;
use graphics::{ActiveShaderProgram, Model};

use crate::modelling::cubic::CalculatedAnimation;
use crate::modelling::{Collision, CubicVertex, ExtendedVertex, Skeleton};

pub trait ExtendedModel: Model
where
    Self::Vertex: ExtendedVertex,
{
}
impl<T> ExtendedModel for T
where
    T: Model,
    T::Vertex: ExtendedVertex,
{
}

pub trait EntityInner: Sized + ExtendedModel
where
    Self::Vertex: CubicVertex,
{
    type BoundingBox<T>: Collision;
    type DrawError<'a>: Error
    where
        Self: 'a;

    fn make_bounding_box<T, F>(&self, depth: usize, init: F) -> Option<Self::BoundingBox<T>>
    where
        F: FnMut(&[(Vector<3>, [Self::Vertex; 3])]) -> T;

    fn skeleton(&self) -> &Skeleton;

    fn box_transform<'a, T>(
        &'a self,
        calc_anim: &'a CalculatedAnimation,
    ) -> <Self::BoundingBox<T> as Collision>::Transform<'a>;

    fn vertex_transform<'a, T>(
        &'a self,
        calc_anim: &'a CalculatedAnimation,
    ) -> <Self::BoundingBox<T> as Collision>::VertexTransform<'a>;

    fn draw<'a, 'b, const OUT: usize, D, L>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'b, '_, 'a, Self, L, D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        calculated_animation: &CalculatedAnimation,
    ) -> Result<(), Self::DrawError<'b>>
    where
        D: FramebufferWithDepth<OUT>;
}
