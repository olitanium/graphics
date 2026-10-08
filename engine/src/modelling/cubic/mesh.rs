use alloc::borrow::Cow;
use alloc::rc::Rc;

mod builder;
pub use builder::Builder as MeshBuilder;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::vertex_array::{self, VertexArray};
use graphics::{ActiveShaderProgram, Model};
use utils::error_boilerplate;

use crate::modelling::CubicVertex;
use crate::modelling::cubic::material::Material;

#[derive(Debug, Clone)]
pub struct Mesh<V: CubicVertex> {
    pub vertex_array: Rc<VertexArray<V>>,
    pub material: Rc<Material>,
}

impl<V: CubicVertex> Mesh<V> {
    pub fn builder() -> MeshBuilder<V> {
        MeshBuilder::new()
    }

    pub const fn new(vertex_array: Rc<VertexArray<V>>, material: Rc<Material>) -> Self {
        Self {
            vertex_array,
            material,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DrawError {
    VertexArrayDraw(vertex_array::DrawError),
}
error_boilerplate!(DrawError);

impl<V: CubicVertex> Mesh<V> {
    pub(crate) fn draw<'b, O, const OUT: usize, D, M>(
        &'b self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'b, M, O, D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
    ) -> Result<(), DrawError>
    where
        D: FramebufferWithDepth<OUT>,
        M: Model<Vertex = V>,
    {
        _ = self.material.register_to(active_shader, "material");

        self.vertex_array
            .draw(active_shader, active_framebuffer)
            .map_err(DrawError::VertexArrayDraw)?;

        Ok(())
    }

    pub fn triangles(&self) -> Cow<'_, [[V; 3]]> {
        self.vertex_array.triangles()
    }
}
