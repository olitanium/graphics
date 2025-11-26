use graphics::ActiveShaderProgram;
use graphics::colour::ColourRGBA;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Vector;

use crate::EngResult;
use crate::modelling::cubic::geometry::{
    AnimationError,
    CalculatedAnimation,
    Pose,
    SkeletalAnimation,
};
use crate::modelling::{BoundingBox, CubicVertex, Skeleton};

mod group;
pub use group::Group as SkeletonBbGroup;

#[derive(Debug, Clone)]
pub struct SkeletonBb<V: CubicVertex, T> {
    per_bone: Vec<BoundingBox<V, 3, T>>,
    skeleton: Skeleton,
}

impl<V: CubicVertex, T> SkeletonBb<V, T> {
    pub fn from_vec_triangle<F: FnMut(&[(Vector<3>, [V; 3])]) -> T>(
        per_mesh: Vec<Vec<[V; 3]>>,
        depth: usize,
        skeleton: Skeleton,
        mut init: F,
    ) -> Option<Self> {
        let opt_per_mesh: Option<Vec<_>> = per_mesh
            .into_iter()
            .map(|mesh| BoundingBox::new_from_triangles(mesh, depth, &mut init))
            .collect();

        Some(Self {
            per_bone: opt_per_mesh?,
            skeleton,
        })
    }
}

impl<V: CubicVertex, T> SkeletonBb<V, T> {
    pub fn overlap<F, O>(
        lhs: &Self,
        lhs_calculated_animation: &CalculatedAnimation,
        lhs_alter_meta: F,

        rhs: &Self,
        rhs_calculated_animation: &CalculatedAnimation,
        rhs_alter_meta: O,
    ) -> Vec<(Vec<[V; 3]>, Vec<[V; 3]>)>
    where
        F: Fn(&mut T),
        O: Fn(&mut T),
    {
        let lhs_alter_meta = &lhs_alter_meta;
        let rhs_alter_meta = &rhs_alter_meta;

        let lhs_calculated_animation = lhs_calculated_animation.poses();
        let rhs_calculated_animation = rhs_calculated_animation.poses();
        lhs.per_bone
            .iter()
            .enumerate()
            .flat_map(|(lhs_bone, lhs_bb)| {
                let lhs_pose =
                    // TODO: well formed
                    *lhs_calculated_animation.get(lhs_bone).unwrap();
                rhs.per_bone
                    .iter()
                    .enumerate()
                    .flat_map(move |(rhs_bone, rhs_bb)| {
                        let rhs_pose = *rhs_calculated_animation.get(rhs_bone).unwrap();

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

    pub(crate) fn draw<NEVER, D: FramebufferWithDepth<1>, F: Fn(&T) -> ColourRGBA>(
        &self,
        level: usize,
        active_shader: &mut ActiveShaderProgram<'_, '_, '_, Vector<3>, (), D::Tex, 1>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, 1, D>,
        calculated_animation: CalculatedAnimation,
        colour: F,
    ) -> EngResult<(), NEVER> {
        for (bone, bbox) in self.per_bone.iter().enumerate() {
            let pose = calculated_animation.get(bone)?;
            bbox.draw(level, pose, active_shader, active_framebuffer, &colour)?
        }

        Ok(())
    }
}
