use graphics::Model;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::shader_program::{ActiveShaderProgram, CullFace};
use graphics::texture::{CubeMap, Texture};
use graphics::vertex_array::VertexArray;

use crate::EngResult;
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

    pub(crate) fn draw<'a, NEVER, const OUT: usize, O, D: FramebufferWithDepth<OUT>>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, SkyBox, O, D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
    ) -> EngResult<(), NEVER> {
        active_shader.cull_face(CullFace::DoNotCull)?;
        active_shader.register_texture(Some(("skybox".to_string(), &self.texture as &dyn Texture)));

        self.model.draw(active_shader, active_framebuffer)?;

        Ok(())
    }
}

impl Model for SkyBox {
    type Vertex = SkyBoxVertex;
}
