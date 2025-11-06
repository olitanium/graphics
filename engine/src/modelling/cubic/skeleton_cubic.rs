use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::shader_program::CullFace;
use graphics::{ActiveShaderProgram, ApiResult};

use crate::EngResult;
use crate::modelling::cubic::geometry::{Pose, SkeletalAnimation};
use crate::modelling::cubic::{Mesh, skeleton};
use crate::modelling::{SkeletalVertex, Skeleton};

mod group;
pub use group::Group as SkeletonCubicGroup;

#[derive(Debug, Clone, Default)]
pub struct SkeletonCubic {
    meshes: Vec<Mesh<SkeletalVertex>>,
    skeleton: Skeleton,
    cull_face: CullFace,
}

impl SkeletonCubic {
    pub fn debug_new(meshes: Vec<Mesh<SkeletalVertex>>, skeleton: Skeleton) -> Self {
        Self {
            meshes,
            skeleton,
            cull_face: CullFace::DoNotCull,
        }
    }

    pub(crate) fn draw<'a, NEVER, const OUT: usize, D: FramebufferWithDepth<OUT>, L>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, SkeletalVertex, (Self, L), D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        root: Pose,
        animation: &SkeletalAnimation,
        time: f32,
    ) -> EngResult<(), NEVER> {
        // Ignore cull_face error
        let _: ApiResult<_, !> = active_shader.cull_face(self.cull_face);

        let matrices = self.skeleton.get_all_bones_matrix(root, animation, time);
        active_shader.set_uniform("bones", &matrices)?;

        for mesh in &self.meshes {
            mesh.draw(active_shader, active_framebuffer)?;
        }

        Ok(())
    }
}
