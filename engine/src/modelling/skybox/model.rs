use std::iter;

use graphics::Model;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::shader_program::{ActiveShaderProgram, CullFace, FaceCullingForcedByProgramError};
use graphics::texture::{CubeMap, Texture};
use graphics::vertex_array::{VertexArray, VertexArrayDrawError};
use utils::error_boilerplate;

use crate::modelling::skybox::SkyBoxVertex;
use crate::modelling::{ExtendedVertex, test_models};

#[derive(Debug)]
pub struct SkyBox {
    pub model: VertexArray<SkyBoxVertex>,
    pub texture: CubeMap,
}

impl SkyBox {
    pub fn new(texture: CubeMap) -> Self {
        let mut model_builder = VertexArray::builder();
        model_builder.extend_triangles(
            test_models::origin_unit_cube().filter_map(ExtendedVertex::from_incomplete_triangle),
        );
        let model = model_builder.build();
        Self { model, texture }
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DrawError<'a> {
    FaceCulling(FaceCullingForcedByProgramError),
    VertexArrayDraw(VertexArrayDrawError<'a>),
}

error_boilerplate!(<'a> DrawError<'a>);

impl SkyBox {
    pub(crate) fn draw<'a, 'b, const OUT: usize, O, D: FramebufferWithDepth<OUT>>(
        &'b self,
        active_shader: &mut ActiveShaderProgram<'a, '_, 'b, SkyBox, O, D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
    ) -> Result<(), DrawError<'a>> {
        active_shader
            .cull_face(CullFace::DoNotCull)
            .map_err(|err| DrawError::FaceCulling(err))?;
        active_shader.register_texture(iter::once((
            "skybox".to_string(),
            &self.texture as &dyn Texture,
        )));

        self.model
            .draw(active_shader, active_framebuffer)
            .map_err(|err| DrawError::VertexArrayDraw(err))?;

        Ok(())
    }
}

impl Model for SkyBox {
    type Vertex = SkyBoxVertex;
}
