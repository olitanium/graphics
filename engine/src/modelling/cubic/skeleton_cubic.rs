use std::fmt::Debug;
use std::{convert, iter};

use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::shader_program::CullFace;
use graphics::{ActiveShaderProgram, Model};
use utils::{error_boilerplate, partition};

use crate::entity::EntityInner;
use crate::modelling::cubic::geometry::{
    CalculatedAnimation,
    SignaturesDoNotMatchError,
    SkeletalAnimation,
};
use crate::modelling::cubic::{Mesh, MeshDrawError, SkeletonBoneIndex};
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
}

#[derive(Debug)]
pub enum DrawError<'a> {
    SignaturesDoNotMatch(SignaturesDoNotMatchError),
    Mesh(MeshDrawError<'a>),
}
error_boilerplate!(<'a> DrawError<'a>);

impl SkeletonCubic {
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
            .map_err(|err| DrawError::SignaturesDoNotMatch(err))?;

        let matrices: Vec<_> = poses
            .iter()
            .map(|pose| pose.as_transformation_matrix())
            .collect();
        _ = active_shader.set_uniform("bones", &matrices);

        let matrices_quat: Vec<_> = poses
            .iter()
            .map(|pose| pose.as_matrix_for_shader())
            .collect();
        _ = active_shader.set_uniform("bones_quat", &matrices_quat);

        for mesh in &self.meshes {
            mesh.draw(active_shader, active_framebuffer)
                .map_err(|err| DrawError::Mesh(err))?;
        }

        Ok(())
    }
}

impl EntityInner for SkeletonCubic {
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

        SkeletonBb::from_vec_triangle(per_mesh, depth, init)
    }

    fn skeleton(&self) -> &Skeleton {
        &self.skeleton
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

impl Model for SkeletonCubic {
    type Vertex = SkeletalVertex;
}
