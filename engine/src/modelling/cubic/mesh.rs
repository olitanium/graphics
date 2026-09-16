use std::borrow::Cow;
use std::rc::Rc;

mod builder;
pub use builder::Builder as MeshBuilder;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::vertex_array::{VertexArray, VertexArrayDrawError};
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
pub enum DrawError<'a> {
    VertexArrayDraw(VertexArrayDrawError<'a>),
}
error_boilerplate!(<'a> DrawError<'a>);

impl<V: CubicVertex> Mesh<V> {
    pub(crate) fn draw<
        'a,
        'b,
        M: Model<Vertex = V>,
        O,
        const OUT: usize,
        D: FramebufferWithDepth<OUT>,
    >(
        &'b self,
        active_shader: &mut ActiveShaderProgram<'a, '_, 'b, M, O, D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
    ) -> Result<(), DrawError<'a>> {
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
