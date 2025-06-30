use crate::buffers::framebuffer::{ActiveFramebuffer, FramebufferWithDepth};
use crate::buffers::vertex_array::VertexArray;
use crate::modelling::cubic::SimpleVertex;
use crate::shader_program::{ActiveShaderProgram, CullFace};
use crate::texture::{CubeMap, Texture};
use crate::Result;

#[derive(Debug)]
pub struct SkyBox {
    pub model: VertexArray<SimpleVertex>,
    pub texture: CubeMap,
}

impl SkyBox {
    pub fn new(texture: CubeMap) -> Self {
        Self {
            model: VertexArray::cube(1.0),
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
