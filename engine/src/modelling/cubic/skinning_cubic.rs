use core::fmt::Debug;
use core::iter;

use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Vector;
use graphics::shader_program::{CullFace, ShaderSetUniformError};
use graphics::{ActiveShaderProgram, Model};
use utils::error_boilerplate;

use crate::entity::EntityInner;
use crate::modelling::cubic::{
    CalculatedAnimation,
    Mesh,
    MeshDrawError,
    SignaturesDoNotMatchError,
    SkeletalAnimation,
    SkeletonBoneIndex,
};
use crate::modelling::{Collision, SkeletalVertex, Skeleton, SkeletonBb, test_models};

mod group;
pub use group::Group;

#[derive(Debug, Clone, Default)]
pub struct SkinningCubic {
    meshes: Vec<Mesh<SkeletalVertex>>,
    pub skeleton: Skeleton,
    pub cull_face: CullFace,
}

impl SkinningCubic {
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
}

#[derive(Debug)]
#[non_exhaustive]
pub enum DrawError {
    SignaturesDoNotMatch(SignaturesDoNotMatchError),
    Mesh(MeshDrawError),
    SetUniform {
        mat: ShaderSetUniformError,
        quat: ShaderSetUniformError,
    },
}
error_boilerplate!(DrawError);

impl SkinningCubic {
    pub(crate) fn draw<'a, const OUT: usize, D, L>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, Self, L, D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        calculated_animation: &CalculatedAnimation,
    ) -> Result<(), DrawError>
    where
        D: FramebufferWithDepth<OUT>,
    {
        // Ignore cull_face error
        _ = active_shader.cull_face(self.cull_face);

        let poses = calculated_animation
            .poses(&self.skeleton)
            .map_err(DrawError::SignaturesDoNotMatch)?;

        let matrices: Vec<_> = poses
            .iter()
            .map(|pose| pose.as_transformation_matrix())
            .collect();
        let matrices_res = active_shader.set_uniform(&c"bones", matrices.as_slice());

        let matrices_quat: Vec<_> = poses
            .iter()
            .map(|pose| pose.as_matrix_for_shader())
            .collect();
        let quat_res = active_shader.set_uniform(&c"bones_quat", matrices_quat.as_slice());

        if let (Err(mat_err), Err(quat_err)) = (matrices_res, quat_res) {
            return Err(DrawError::SetUniform {
                mat: mat_err,
                quat: quat_err,
            });
        }

        for mesh in &self.meshes {
            mesh.draw(active_shader, active_framebuffer)
                .map_err(DrawError::Mesh)?;
        }

        Ok(())
    }
}

impl EntityInner for SkinningCubic {
    type BoundingBox<T> = SkeletonBb<SkeletalVertex, T>;
    type DrawError = DrawError;

    fn make_bounding_box<T, F>(&self, depth: usize, init: F) -> Option<Self::BoundingBox<T>>
    where
        F: FnMut(&[(Vector<3>, [Self::Vertex; 3])]) -> T,
    {
        let per_mesh = partition::partition(
            self.triangles(),
            |&tri| {
                tri.into_iter().flat_map(|vertex| {
                    #[expect(
                        clippy::cast_sign_loss,
                        reason = "index would ideally be u32, will fix when I have worked out \
                                  `SetAttribIPointer`"
                    )]
                    iter::zip(vertex.bone_indices, vertex.bone_weights)
                        .filter_map(|(index, weight)| (weight != 0.0).then_some(index as usize))
                })
            },
            Clone::clone,
        );

        SkeletonBb::from_vec_triangle(per_mesh, depth, init)
    }

    fn skeleton(&self) -> &Skeleton {
        &self.skeleton
    }

    fn vertex_transform<'a, T>(
        &'a self,
        _calc_anim: &'a CalculatedAnimation,
    ) -> <Self::BoundingBox<T> as Collision>::VertexTransform<'a> {
    }

    fn box_transform<'a, T>(
        &'a self,
        calc_anim: &'a CalculatedAnimation,
    ) -> <Self::BoundingBox<T> as Collision>::Transform<'a> {
        (&self.skeleton, calc_anim)
    }

    fn draw<'a, const OUT: usize, D, L>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'_, '_, 'a, Self, L, D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        calculated_animation: &CalculatedAnimation,
    ) -> Result<(), Self::DrawError>
    where
        D: FramebufferWithDepth<OUT>,
    {
        self.draw(active_shader, active_framebuffer, calculated_animation)
    }
}

impl Model for SkinningCubic {
    type Vertex = SkeletalVertex;
}
