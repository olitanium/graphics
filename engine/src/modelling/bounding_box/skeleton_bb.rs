use graphics::ActiveShaderProgram;
use graphics::colour::ColourRGBA;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Vector;

use crate::EngResult;
use crate::modelling::cubic::geometry::{Pose, SkeletalAnimation};
use crate::modelling::{BoundingBox, SimpleCubic, CubicVertex, SimpleVertex, Skeleton};

mod group;
pub use group::Group as SkeletonBbGroup;

#[derive(Debug)]
pub struct SkeletonBb<V: CubicVertex, T> {
    per_bone: Vec<BoundingBox<V, 3, T>>,
}

impl<T> SkeletonBb<SimpleVertex, T> {
    pub fn from_cubic_simplevertex<F: FnMut(&[(Vector<3>, [SimpleVertex; 3])]) -> T>(
        cubic: &SimpleCubic,
        depth: usize,
        mut init: F,
    ) -> Option<Self> {
        let per_mesh = partition(cubic.triangles(), |(bone, _)| vec![*bone], |(_, tri)| tri);

        let opt_per_mesh: Option<Vec<_>> = per_mesh
            .into_iter()
            .map(|mesh| BoundingBox::new_from_triangles(mesh, depth, &mut init))
            .collect();

        Some(Self {
            per_bone: opt_per_mesh?,
        })
    }
}

impl<V: CubicVertex, T> SkeletonBb<V, T> {
    pub fn overlap<F, O>(
        lhs: &Self,
        lhs_skeleton: &Skeleton,
        lhs_pose: Pose,
        lhs_animation: &SkeletalAnimation,
        lhs_time: f32,
        lhs_alter_meta: F,

        rhs: &Self,
        rhs_skeleton: &Skeleton,
        rhs_pose: Pose,
        rhs_animation: &SkeletalAnimation,
        rhs_time: f32,
        rhs_alter_meta: O,
    ) -> Vec<(Vec<[V; 3]>, Vec<[V; 3]>)>
    where
        F: Fn(&mut T),
        O: Fn(&mut T),
    {
        let lhs_alter_meta = &lhs_alter_meta;
        let rhs_alter_meta = &rhs_alter_meta;

        lhs.per_bone
            .iter()
            .enumerate()
            .flat_map(|(lhs_bone, lhs_bb)| {
                let lhs_pose =
                    lhs_skeleton.get_bone_pose(lhs_pose, true, lhs_bone, lhs_animation, lhs_time).unwrap(/*TODO: Animation well formed */);
                rhs.per_bone
                    .iter()
                    .enumerate()
                    .flat_map(move |(rhs_bone, rhs_bb)| {
                        let rhs_pose = rhs_skeleton.get_bone_pose(
                            rhs_pose,
                            true,
                            rhs_bone,
                            rhs_animation,
                            rhs_time,
                        ).unwrap(/*TODO: Animation well formed */);

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

    pub(crate) fn draw<D: FramebufferWithDepth<1>, F: Fn(&T) -> ColourRGBA>(
        &self,
        level: usize,
        skeleton: &Skeleton,
        active_shader: &mut ActiveShaderProgram<'_, '_, '_, Vector<3>, (), D::Tex, 1>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, 1, D>,
        pose: Pose,
        animation: &SkeletalAnimation,
        time: f32,
        colour: F,
    ) -> EngResult<(), !> {
        for (bone, bbox) in self.per_bone.iter().enumerate() {
            let pose = skeleton.get_bone_pose(pose, true, bone, animation, time).unwrap(/* TODO: Well formed animation */);
            bbox.draw(level, pose, active_shader, active_framebuffer, &colour)?
        }

        Ok(())
    }
}

fn partition<
    U,
    T: Clone,
    I: IntoIterator<Item = T>,
    F: FnMut(&T) -> Vec<usize>,
    O: FnMut(T) -> U,
>(
    iter: I,
    mut part_fn: F,
    mut manipulate_elem: O,
) -> Vec<Vec<U>> {
    let mut out = vec![];

    for item in iter {
        let indices = part_fn(&item);
        if let Some(&max) = indices.iter().max() {
            conditionally_resize_vector(&mut out, max + 1, Vec::new);

            for value in indices {
                out[value].push(manipulate_elem(item.clone()))
            }
        }
    }

    out
}

fn conditionally_resize_vector<T, F: FnMut() -> T>(vec: &mut Vec<T>, len: usize, new: F) {
    if vec.len() < len {
        vec.resize_with(len, new);
    }
}
