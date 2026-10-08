use core::iter;

use graphics::Model;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::shader_program::{ActiveShaderProgram, CullFace, FaceCullingForcedByProgramError};
use graphics::texture::{CubeMap, Texture};
use graphics::vertex_array::{self, VertexArray};
use utils::error_boilerplate;

use crate::modelling::skybox::SkyBoxVertex;
use crate::modelling::{ExtendedVertex as _, test_models};

#[derive(Debug)]
pub struct SkyBox {
    pub model: VertexArray<SkyBoxVertex>,
    pub texture: CubeMap,
}

impl SkyBox {
    pub fn new(texture: CubeMap) -> Self {
        let mut model_builder = VertexArray::builder();
        model_builder.extend_triangles(
            test_models::origin_unit_cube()
                .filter_map(|tri| SkyBoxVertex::from_incomplete_triangle(tri).ok()),
        );
        let model = model_builder.build();
        Self { model, texture }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DrawError {
    FaceCulling(FaceCullingForcedByProgramError),
    VertexArrayDraw(vertex_array::DrawError),
}

error_boilerplate!(DrawError);

impl SkyBox {
    pub(crate) fn draw<'b, const OUT: usize, O, D>(
        &'b self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'b, Self, O, D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
    ) -> Result<(), DrawError>
    where
        D: FramebufferWithDepth<OUT>,
    {
        active_shader
            .cull_face(CullFace::DoNotCull)
            .map_err(DrawError::FaceCulling)?;
        active_shader.register_texture(iter::once((
            c"skybox".to_owned(),
            &self.texture as &dyn Texture,
        )));

        self.model
            .draw(active_shader, active_framebuffer)
            .map_err(DrawError::VertexArrayDraw)?;

        Ok(())
    }
}

impl Model for SkyBox {
    type Vertex = SkyBoxVertex;
}
