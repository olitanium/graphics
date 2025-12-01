use std::fmt::Debug;
use std::{convert, iter};

use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::shader_program::CullFace;
use graphics::{ActiveShaderProgram, ApiResult};
use utils::partition;

use crate::EngResult;
use crate::entity::EntityInner;
use crate::modelling::cubic::geometry::{CalculatedAnimation, SkeletalAnimation};
use crate::modelling::cubic::{Mesh, SkeletonBoneIndex};
use crate::modelling::{SkeletalVertex, Skeleton, SkeletonBb, test_models};

mod group;
pub use group::Group as SkeletonCubicGroup;

#[derive(Debug, Clone, Default)]
pub struct SkeletonCubic {
    meshes: Vec<Mesh<SkeletalVertex>>,
    pub skeleton: Skeleton,
    pub cull_face: CullFace,
}

impl SkeletonCubic {
    pub const fn debug_new(meshes: Vec<Mesh<SkeletalVertex>>, skeleton: Skeleton) -> Self {
        Self {
            meshes,
            skeleton,
            cull_face: CullFace::DoNotCull,
        }
    }

    pub fn debug_tail() -> (Self, SkeletonBoneIndex, SkeletalAnimation) {
        test_models::tail_skeleton_cubic(10)
    }

    pub fn triangles(&self) -> Vec<[SkeletalVertex; 3]> {
        self.meshes
            .iter()
            .flat_map(|mesh| mesh.triangles().into_owned())
            .collect()
    }

    pub(crate) fn draw<'a, NEVER, const OUT: usize, D: FramebufferWithDepth<OUT>, L>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, SkeletalVertex, (Self, L), D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        calculated_animation: &CalculatedAnimation,
    ) -> EngResult<(), NEVER> {
        // Ignore cull_face error
        let _: ApiResult<_, !> = active_shader.cull_face(self.cull_face);

        let poses = calculated_animation.poses(&self.skeleton)?;

        let matrices: Vec<_> = poses
            .iter()
            .map(|pose| pose.as_transformation_matrix())
            .collect();
        let _: ApiResult<_, !> = active_shader.set_uniform("bones", &matrices);

        let matrices_quat: Vec<_> = poses
            .iter()
            .map(|pose| pose.as_matrix_for_shader())
            .collect();
        let _: ApiResult<_, !> = active_shader.set_uniform("bones_quat", &matrices_quat);

        for mesh in &self.meshes {
            mesh.draw(active_shader, active_framebuffer)?;
        }

        Ok(())
    }
}

impl EntityInner for SkeletonCubic {
    type BoundingBox<T> = SkeletonBb<SkeletalVertex, T>;
    type Vertex = SkeletalVertex;

    fn make_bounding_box<
        T,
        F: FnMut(&[(graphics::linear_algebra::Vector<3>, [Self::Vertex; 3])]) -> T,
    >(
        &self,
        depth: usize,
        init: F,
    ) -> Option<Self::BoundingBox<T>> {
        let per_mesh = partition(
            self.triangles(),
            |tri| {
                tri.iter()
                    .flat_map(|vertex| {
                        iter::zip(vertex.bone_indices, vertex.bone_weights)
                            .filter_map(|(index, weight)| (weight != 0.0).then_some(index as usize))
                    })
                    .collect()
            },
            convert::identity,
        );

        SkeletonBb::from_vec_triangle(per_mesh, depth, self.skeleton.clone(), init)
    }

    fn skeleton(&self) -> &Skeleton {
        &self.skeleton
    }

    fn draw<'a, NEVER, const OUT: usize, D: FramebufferWithDepth<OUT>, L>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, Self::Vertex, (Self, L), D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        calculated_animation: &CalculatedAnimation,
    ) -> EngResult<(), NEVER> {
        self.draw(active_shader, active_framebuffer, calculated_animation)
    }
}
