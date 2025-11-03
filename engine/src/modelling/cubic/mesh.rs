use std::borrow::Cow;
use std::rc::Rc;

mod builder;
pub use builder::Builder as MeshBuilder;
use graphics::ActiveShaderProgram;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::vertex_array::VertexArray;

use crate::EngResult;
use crate::modelling::{CubicVertex, ExtendedVertex};
use crate::modelling::cubic::material::Material;

#[derive(Debug, Clone)]
pub struct Mesh<V: CubicVertex> {
    pub vertex_array: Rc<VertexArray<V>>,
    pub material: Rc<Material>,
}

impl<V: CubicVertex> Mesh<V> {
    pub fn builder() -> MeshBuilder {
        MeshBuilder::new()
    }

    pub fn new(vertex_array: Rc<VertexArray<V>>, material: Rc<Material>) -> Self {
        Self {
            vertex_array,
            material,
        }
    }
}

impl<V: CubicVertex> Mesh<V>
where
    [(); V::ELEMENT_COUNT]:,
{
    pub(crate) fn draw<'a, const OUT: usize, D: FramebufferWithDepth<OUT>, M>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, V, M, D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
    ) -> EngResult<(), !> {
        self.material.register_to(active_shader, "material");

        self.vertex_array.draw(active_shader, active_framebuffer)?;

        Ok(())
    }

    pub fn triangles(&self) -> Cow<'_, [[V; 3]]> {
        self.vertex_array.triangles()
    }
}