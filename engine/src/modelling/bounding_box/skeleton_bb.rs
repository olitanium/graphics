use graphics::ActiveShaderProgram;
use graphics::colour::ColourRGBA;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Vector;
use graphics::texture::FlatTexture;
use graphics::vertex_array::VertexArray;
use utils::error_boilerplate;

// use crate::modelling::bounding_box::collision::TempModelTransform;
use crate::modelling::bounding_box::element::DrawRecursiveError;
use crate::modelling::cubic::SkeletonBoneIndex;
use crate::modelling::cubic::geometry::{CalculatedAnimation, GetAnimationError};
use crate::modelling::{BoundingBox, Collision, CubicVertex, Skeleton};

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

    pub fn debug_get_first(&self) -> &BoundingBox<V, 3, T> {
        self.per_bone.first().unwrap()
    }
}

// impl<V: CubicVertex, T> SkeletonBb<V, T> {
// pub fn detect_collision<F, O>(
// lhs: &Self,
// lhs_calculated_animation: &CalculatedAnimation,
// lhs_skeleton: &Skeleton,
// lhs_alter_meta: F,
//
// rhs: &Self,
// rhs_calculated_animation: &CalculatedAnimation,
// rhs_skeleton: &Skeleton,
// rhs_alter_meta: O,
// ) -> bool
// where
// F: Fn(&SkeletonBoneIndex, &mut T),
// O: Fn(&SkeletonBoneIndex, &mut T),
// {
// let lhs_alter_meta = &lhs_alter_meta;
// let rhs_alter_meta = &rhs_alter_meta;
//
// const ERROR_MSG: &str = "CalulatedAnimation has been generated from this
// skeleton"; lhs.per_bone
// .iter()
// .enumerate()
// .any(move |(lhs_bone, lhs_bb)| {
// HACK: making SkeletonBoneIndex outside the skeleton should be illegal
// let lhs_bone = &SkeletonBoneIndex(lhs_bone);
// let lhs_pose = lhs_calculated_animation
// .get(lhs_bone, lhs_skeleton)
// .expect(ERROR_MSG);
// rhs.per_bone
// .iter()
// .enumerate()
// .any(move |(rhs_bone, rhs_bb)| {
// HACK: making SkeletonBoneIndex outside the skeleton should be illegal
// let rhs_bone = &SkeletonBoneIndex(rhs_bone);
// let rhs_pose = rhs_calculated_animation
// .get(rhs_bone, rhs_skeleton)
// .expect(ERROR_MSG);
//
// BoundingBox::detect_collision(
// lhs_bb,
// lhs_pose,
// |t| lhs_alter_meta(lhs_bone, t),
// rhs_bb,
// rhs_pose,
// |t| rhs_alter_meta(rhs_bone, t),
// )
// })
// })
// }
// }

#[derive(Debug)]
pub enum DrawError<'a> {
    DrawRecursive(DrawRecursiveError<'a>),
    GetAnimation(GetAnimationError),
}

error_boilerplate!(<'a> DrawError<'a>);

impl<V: CubicVertex, T> SkeletonBb<V, T> {
    pub(crate) fn draw<
        'a,
        D: FramebufferWithDepth<1, Tex = FlatTexture>,
        F: FnMut() -> ColourRGBA,
    >(
        &self,
        level: usize,
        active_shader: &mut ActiveShaderProgram<'a, '_, '_, VertexArray<Vector<3>>, (), D::Tex, 1>,
        active_framebuffer: &mut ActiveFramebuffer<'_, '_, 1, D>,
        skeleton: &Skeleton,
        calculated_animation: CalculatedAnimation,
        mut colour: F,
    ) -> Result<(), DrawError<'a>> {
        for (bone, bbox) in self.per_bone.iter().enumerate() {
            let bone = &SkeletonBoneIndex(bone);

            let pose = calculated_animation
                .get(bone, skeleton)
                .map_err(DrawError::GetAnimation)?;
            bbox.draw(level, pose, active_shader, active_framebuffer, &mut colour)
                .map_err(DrawError::DrawRecursive)?
        }

        Ok(())
    }
}

impl<VThis: CubicVertex, TThis> Collision for SkeletonBb<VThis, TThis> {
    type FuncInputs = SkeletonBoneIndex;
    type Meta = TThis;
    type Transform<'a> = (&'a Skeleton, &'a CalculatedAnimation);

    fn collides<
        T: Collision,
        F: FnMut(Self::FuncInputs, &mut Self::Meta, T::FuncInputs, &mut T::Meta),
    >(
        &self,
        transform: Self::Transform<'_>,
        target: &T,
        target_transform: T::Transform<'_>,
        mut alter_meta: F,
    ) -> bool
    where
        for<'a> T::Transform<'a>: Clone,
    {
        self.per_bone
            .iter()
            .enumerate()
            .any(|(index, self_bounding_box)| {
                let skeleton_index = SkeletonBoneIndex(index);
                let Ok(this_transform) = transform.1.get(&skeleton_index, transform.0) else {
                    return false;
                };

                self_bounding_box.collides(
                    this_transform,
                    target,
                    target_transform.clone(),
                    |(), self_meta, target_inputs, target_meta| {
                        alter_meta(
                            skeleton_index.clone_inner(),
                            self_meta,
                            target_inputs,
                            target_meta,
                        )
                    },
                )
            })
    }
}

impl<V: CubicVertex, T> From<BoundingBox<V, 3, T>> for SkeletonBb<V, T> {
    fn from(value: BoundingBox<V, 3, T>) -> Self {
        Self {
            per_bone: vec![value],
        }
    }
}

// impl<'a> TempModelTransform for (&'a Skeleton, &'a CalculatedAnimation) {
// fn get_root_pose(&self) -> crate::modelling::cubic::geometry::Pose {
// self.1.root()
// }
// }
