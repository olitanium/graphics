use graphics::ActiveShaderProgram;
use graphics::colour::ColourRGBA;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Vector;
use graphics::texture::FlatTexture;
use graphics::vertex_array::VertexArray;

use crate::modelling::bounding_box::element::DrawRecurisveError;
use crate::modelling::cubic::SkeletonBoneIndex;
use crate::modelling::cubic::geometry::{CalculatedAnimation, GetAnimationError};
use crate::modelling::{BoundingBox, CubicVertex, Skeleton};

mod group;
pub use group::Group as SkeletonBbGroup;

#[derive(Debug, Clone)]
pub struct SkeletonBb<V: CubicVertex, T> {
    per_bone: Vec<BoundingBox<V, 3, T>>,
}

impl<V: CubicVertex, T> SkeletonBb<V, T> {
    pub fn from_vec_triangle<F: FnMut(&[(Vector<3>, [V; 3])]) -> T>(
        per_mesh: Vec<Vec<[V; 3]>>,
        depth: usize,
        mut init: F,
    ) -> Option<Self> {
        let opt_per_mesh: Option<Vec<_>> = per_mesh
            .into_iter()
            .map(|mesh| BoundingBox::new_from_triangles(mesh, depth, &mut init))
            .collect();

        Some(Self {
            per_bone: opt_per_mesh?,
        })
    }

    pub fn from_single(bbox: BoundingBox<V, 3, T>) -> Self {
        Self {
            per_bone: vec![bbox],
        }
    }
}

impl<V: CubicVertex, T> SkeletonBb<V, T> {
    pub fn detect_collision<F, O>(
        lhs: &Self,
        lhs_calculated_animation: &CalculatedAnimation,
        lhs_skeleton: &Skeleton,
        lhs_alter_meta: F,

        rhs: &Self,
        rhs_calculated_animation: &CalculatedAnimation,
        rhs_skeleton: &Skeleton,
        rhs_alter_meta: O,
    ) -> Vec<(Vec<[V; 3]>, Vec<[V; 3]>)>
    where
        F: Fn(&mut T),
        O: Fn(&mut T),
    {
        let lhs_alter_meta = &lhs_alter_meta;
        let rhs_alter_meta = &rhs_alter_meta;

        const ERROR_MSG: &str = "CalulatedAnimation has been generated from this skeleton";
        // let lhs_calculated_animation = lhs_calculated_animation
        // .poses(lhs_skeleton)
        // .expect(ERROR_MSG);
        // let rhs_calculated_animation = rhs_calculated_animation
        // .poses(rhs_skeleton)
        // .expect(ERROR_MSG);
        lhs.per_bone
            .iter()
            .enumerate()
            .flat_map(|(lhs_bone, lhs_bb)| {
                // HACK: making SkeletonBoneIndex outside the skeleton should be illegal
                let lhs_bone = SkeletonBoneIndex(lhs_bone);
                let lhs_pose = lhs_calculated_animation
                    .get(lhs_bone, lhs_skeleton)
                    .expect(ERROR_MSG);
                rhs.per_bone
                    .iter()
                    .enumerate()
                    .flat_map(move |(rhs_bone, rhs_bb)| {
                        // HACK: making SkeletonBoneIndex outside the skeleton should be illegal
                        let rhs_bone = SkeletonBoneIndex(rhs_bone);
                        let rhs_pose = rhs_calculated_animation
                            .get(rhs_bone, rhs_skeleton)
                            .expect(ERROR_MSG);

                        BoundingBox::detect_collision(
                            lhs_bb,
                            lhs_pose,
                            lhs_alter_meta,
                            rhs_bb,
                            rhs_pose,
                            rhs_alter_meta,
                        )
                    })
            })
            .collect()
    }
}

#[derive(Debug)]
pub enum DrawError<'a> {
    DrawRecursive(DrawRecurisveError<'a>),
    GetAnimation(GetAnimationError),
}

impl<V: CubicVertex, T> SkeletonBb<V, T> {
    pub(crate) fn draw<
        'a,
        D: FramebufferWithDepth<1, Tex = FlatTexture>,
        F: Fn(&T) -> ColourRGBA,
    >(
        &self,
        level: usize,
        active_shader: &mut ActiveShaderProgram<'a, '_, '_, VertexArray<Vector<3>>, (), D::Tex, 1>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, 1, D>,
        skeleton: &Skeleton,
        calculated_animation: CalculatedAnimation,
        colour: F,
    ) -> Result<(), DrawError<'a>> {
        for (bone, bbox) in self.per_bone.iter().enumerate() {
            // HACK do I want to store the index for "completeness"?
            let bone = SkeletonBoneIndex(bone);

            let pose = calculated_animation
                .get(bone, skeleton)
                .map_err(|err| DrawError::GetAnimation(err))?;
            bbox.draw(level, pose, active_shader, active_framebuffer, &colour)
                .map_err(|err| DrawError::DrawRecursive(err))?
        }

        Ok(())
    }
}
