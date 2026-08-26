use std::fmt::Debug;
use std::iter;

use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
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
use crate::modelling::{SkeletalVertex, Skeleton, SkeletonBb, test_models};

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
pub enum DrawError<'a> {
    SignaturesDoNotMatch(SignaturesDoNotMatchError),
    Mesh(MeshDrawError<'a>),
    SetUniform {
        mat: ShaderSetUniformError<'a>,
        quat: ShaderSetUniformError<'a>,
    },
}
error_boilerplate!(<'a> DrawError<'a>);

impl SkinningCubic {
    pub(crate) fn draw<'b, 'a, const OUT: usize, D: FramebufferWithDepth<OUT>, L>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'b, '_, 'a, Self, L, D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        calculated_animation: &CalculatedAnimation,
    ) -> Result<(), DrawError<'b>> {
        // Ignore cull_face error
        _ = active_shader.cull_face(self.cull_face);

        let poses = calculated_animation
            .poses(&self.skeleton)
            .map_err(DrawError::SignaturesDoNotMatch)?;

        let matrices: Vec<_> = poses
            .iter()
            .map(|pose| pose.as_transformation_matrix())
            .collect();
        let matrices_res = active_shader.set_uniform("bones", matrices.as_slice());

        let matrices_quat: Vec<_> = poses
            .iter()
            .map(|pose| pose.as_matrix_for_shader())
            .collect();
        let quat_res = active_shader.set_uniform("bones_quat", matrices_quat.as_slice());

        match (matrices_res, quat_res) {
            (Err(mat_err), Err(quat_err)) => {
                return Err(DrawError::SetUniform {
                    mat: mat_err,
                    quat: quat_err,
                });
            }
            _ => { //if one or both succeed, we're happy
            }
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
    type DrawError<'a> = DrawError<'a>;

    fn make_bounding_box<
        T,
        F: FnMut(&[(graphics::linear_algebra::Vector<3>, [Self::Vertex; 3])]) -> T,
    >(
        &self,
        depth: usize,
        init: F,
    ) -> Option<Self::BoundingBox<T>> {
        let per_mesh = partition::partition(
            self.triangles(),
            |&tri| {
                tri.into_iter().flat_map(|vertex| {
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
        calc_anim: &'a CalculatedAnimation,
    ) -> <Self::BoundingBox<T> as crate::modelling::Collision>::VertexTransform<'a> {
    }

    fn box_transform<'a, T>(
        &'a self,
        calc_anim: &'a CalculatedAnimation,
    ) -> <Self::BoundingBox<T> as crate::modelling::Collision>::Transform<'a> {
        (&self.skeleton, calc_anim)
    }

    fn draw<'a, 'b, const OUT: usize, D: FramebufferWithDepth<OUT>, L>(
        &'a self,
        active_shader: &mut ActiveShaderProgram<'b, '_, 'a, Self, L, D::Tex, OUT>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, OUT, D>,
        calculated_animation: &CalculatedAnimation,
    ) -> Result<(), Self::DrawError<'b>> {
        self.draw(active_shader, active_framebuffer, calculated_animation)
    }
}

impl Model for SkinningCubic {
    type Vertex = SkeletalVertex;
}
