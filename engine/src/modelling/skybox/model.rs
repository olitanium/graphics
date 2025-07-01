use graphics::buffers::{ActiveFramebuffer, FramebufferWithDepth};
use graphics::buffers::VertexArray;
use crate::modelling::test_models::vertex_array_cube;
use crate::modelling::SimpleVertex;
use graphics::shader_program::{ActiveShaderProgram, CullFace};
use graphics::texture::{CubeMap, Texture};
use graphics::error::Result;

#[derive(Debug)]
pub struct SkyBox {
    pub model: VertexArray<SimpleVertex>,
    pub texture: CubeMap,
}

impl SkyBox {
    pub fn new(texture: CubeMap) -> Self {
        Self {
            model: vertex_array_cube(1.0),
            texture,
        }
    }

    pub(crate) fn draw<'a, const OUT: usize, D: FramebufferWithDepth<OUT>>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, Self, D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
    ) -> Result<()> {
        active_shader.cull_face(CullFace::DoNotCull)?;
        active_shader.register_texture(Some(("skybox".to_string(), &self.texture as &dyn Texture)));

        self.model.draw(active_shader, active_framebuffer)?;

        Ok(())
    }
}
