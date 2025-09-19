use std::borrow::Cow;
use std::rc::Rc;

mod builder;
pub use builder::Builder as MeshBuilder;
use graphics::ActiveShaderProgram;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::vertex_array::VertexArray;

use crate::EngResult;
use crate::modelling::cubic::material::Material;
use crate::modelling::{Cubic, SimpleVertex, Skeleton};

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertex_array: Rc<VertexArray<SimpleVertex>>,
    pub material: Rc<Material>,
    pub bone: usize,
}

impl Mesh {
    pub fn builder() -> MeshBuilder {
        MeshBuilder::new()
    }

    pub fn new(
        vertex_array: Rc<VertexArray<SimpleVertex>>,
        material: Rc<Material>,
        bone: usize,
    ) -> Self {
        Self {
            vertex_array,
            material,
            bone,
        }
    }

    pub(crate) fn draw<'a, const OUT: usize, D: FramebufferWithDepth<OUT>, L>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, (Cubic, L), D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        skeleton: &Skeleton,
        relative: bool,
        animation: usize,
        time: f32,
    ) -> EngResult<(), !> {
        self.material.register_to(active_shader, "material");

        active_shader.set_uniform(
            "model".into(),
            &skeleton
                .get_bone_pose(relative, self.bone, animation, time)
                .as_matrix(),
        );

        self.vertex_array.draw(active_shader, active_framebuffer)?;

        Ok(())
    }

    pub fn triangles(&self) -> Cow<'_, [[SimpleVertex; 3]]> {
        self.vertex_array.triangles()
    }

    pub fn triangles_from_vec(vec: &[Self]) -> Vec<[SimpleVertex; 3]> {
        vec.into_iter()
            .map(|mesh| mesh.triangles().into_owned())
            .flatten()
            .collect()
    }
}
