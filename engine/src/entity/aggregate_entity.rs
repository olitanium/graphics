use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::{iter, mem};

use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Vector;
use graphics::{ActiveShaderProgram, Model};
use utils::error_boilerplate;

use crate::modelling::cubic::SkeletonBoneIndex;
use crate::modelling::cubic::geometry::{
    AggregateCalcAnim,
    BuildFixingBoneError,
    CalcAnimUnitIndex,
    CalculatedAnimation,
    Pose,
    PushAnimationError,
    SignaturesDoNotMatchError,
    SkeletalAnimation,
};
use crate::modelling::cubic::lighting::simple::ListLights;
use crate::modelling::{Collision, CubicVertex, Skeleton};

mod group;
pub use group::Group as AggregateGroup;

mod error;
pub use error::PushModelError;

mod from_to;
pub use from_to::FromToAggregateEntity;

mod entity_inner;
pub use entity_inner::EntityInner;

pub struct AggregateEntity<T, M: EntityInner>
where
    M::Vertex: CubicVertex,
{
    base_model: M,
    base_bounding_box: Option<M::BoundingBox<T>>,
    pub base_pose: Pose,
    children: Vec<Option<AggregateUnit<T, M>>>,
}

impl<T: Debug, M: EntityInner + Debug> Debug for AggregateEntity<T, M>
where
    M::Vertex: CubicVertex,
    M::BoundingBox<T>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(AggregateEntity))
            .field("base_model", &self.base_model)
            .field("base_bounding_box", &self.base_bounding_box)
            .field("base_pose", &self.base_pose)
            .field("children", &self.children)
            .finish()
    }
}

#[derive(Debug)]
struct AggregateUnit<T, M: EntityInner>
where
    M::Vertex: CubicVertex,
{
    model: M,
    bounding_box: Option<M::BoundingBox<T>>,
    offset: Pose,
    skeleton_attachment_point: SkeletonBoneIndex,
    parent_unit: AggregateModelIndex,
    children: Vec<AggregateModelIndex>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use = "AggregateModelIndex is a handle into the entity, it is used to set animations and as \
              an attachment point for further units"]
pub struct AggregateModelIndex(pub(crate) usize);

impl AggregateModelIndex {
    pub const ROOT_MODEL_INDEX: Self = Self(0);
}

impl<T, M: EntityInner> AggregateEntity<T, M>
where
    M::Vertex: CubicVertex,
{
    pub fn new<F: FnMut(&[(Vector<3>, [M::Vertex; 3])]) -> T>(
        base_model: M,
        depth: usize,
        init: F,
    ) -> Self {
        let bounding_box = base_model.make_bounding_box(depth, init);

        Self {
            base_bounding_box: bounding_box,
            base_model,
            base_pose: Pose::IDENTITY,
            children: Vec::new(),
        }
    }
}

impl<T, M: EntityInner> AggregateEntity<T, M>
where
    M::Vertex: CubicVertex,
{
    pub fn push_model<F: FnMut(&[(Vector<3>, [M::Vertex; 3])]) -> T>(
        &mut self,
        other_model: M,
        offset: Pose,
        attachment_point: SkeletonBoneIndex,
        parent_unit: AggregateModelIndex,

        bounding_box_depth: usize,
        bounding_box_init: F,
    ) -> Result<AggregateModelIndex, PushModelError> {
        // check that the parent unit is Some and the bone exists
        self.check_parent_and_bone(attachment_point, parent_unit)?;

        let bounding_box = other_model.make_bounding_box(bounding_box_depth, bounding_box_init);

        let child = AggregateUnit {
            model: other_model,
            bounding_box,
            offset,
            skeleton_attachment_point: attachment_point,
            parent_unit,
            children: Vec::new(),
        };
        // get index of the first None

        let child_at = self.push_aggregate_unit(child);

        Ok(child_at)
    }

    fn check_parent_and_bone(
        &self,
        attachment_point: SkeletonBoneIndex,
        parent_unit: AggregateModelIndex,
    ) -> Result<(), PushModelError> {
        let model = if parent_unit == AggregateModelIndex::ROOT_MODEL_INDEX {
            &self.base_model
        } else {
            &self
                .children
                .get(parent_unit.0 - 1)
                .ok_or(PushModelError::ParentUnitDoesNotExist)?
                .as_ref()
                .ok_or(PushModelError::ParentUnitDoesNotExist)?
                .model
        };

        model
            .skeleton()
            .get(attachment_point)
            .is_bone()
            .ok_or(PushModelError::BoneInParentDoesNotExist)
    }
}

impl<T, M: EntityInner> AggregateEntity<T, M>
where
    M::Vertex: CubicVertex,
{
    fn push_aggregate_unit(&mut self, unit: AggregateUnit<T, M>) -> AggregateModelIndex {
        let parent_unit = unit.parent_unit;
        let child_at = if let Some((index, child_dest)) = self
            .children
            .iter_mut()
            .enumerate()
            // skip here ensures that the parent is always above the child.
            // note that the `self.children` is offset by 1, so if the parent is ROOT (0)
            // no children are skipped, if the parent is 1, only the first (the parent itself) is
            // skipped, etc.
            .skip(unit.parent_unit.0)
            .find(|(_, opt_unit)| opt_unit.is_none())
        {
            *child_dest = Some(unit);
            AggregateModelIndex(index + 1)
        } else {
            self.children.push(Some(unit));
            let output = self.children.len();
            AggregateModelIndex(output)
        };

        if parent_unit != AggregateModelIndex::ROOT_MODEL_INDEX {
            let parent_mut = self
                .children
                .get_mut(parent_unit.0 - 1)
                .expect("parent in aggregate unit must exist")
                .as_mut()
                .expect("parent in aggregate unit must exist");
            parent_mut.children.push(child_at);
        }

        child_at
    }

    pub fn pop(&mut self, index: AggregateModelIndex) {
        if index == AggregateModelIndex::ROOT_MODEL_INDEX {
            return;
        }
        if let Some(Some(unit)) = self.children.get_mut(index.0 - 1) {
            for unit_index in mem::take(&mut unit.children) {
                self.pop(unit_index);
            }
        }
        if let Some(opt_empty_unit) = self.children.get_mut(index.0 - 1) {
            eprintln!("pop: {index:?}");
            *opt_empty_unit = None;
        }
    }
}

#[derive(Debug)]
pub enum CalculateAnimationError {
    BadAnimationInput {
        error: SignaturesDoNotMatchError,
        parent: AggregateModelIndex,
    },
    FixAnimationUnitDoesNotExist,
    FixAnimationBoneDoesNotExist,
}
error_boilerplate!(CalculateAnimationError);

impl<T, M: EntityInner> AggregateEntity<T, M>
where
    M::Vertex: CubicVertex,
{
    pub fn calculate_animation<'a>(
        &'a self,
        fix_anim: Option<(AggregateModelIndex, SkeletonBoneIndex, Pose)>,
        mut animations: HashMap<AggregateModelIndex, (&'a SkeletalAnimation, f32)>,
    ) -> Result<AggregateCalcAnim, CalculateAnimationError> {
        let indices = (1..).map(AggregateModelIndex);
        let first_animation = animations.remove(&AggregateModelIndex::ROOT_MODEL_INDEX);

        let mut anim_builder =
            AggregateCalcAnim::builder(self.base_pose, self.base_model.skeleton(), first_animation)
                .map_err(|error| CalculateAnimationError::BadAnimationInput {
                    error,
                    parent: AggregateModelIndex::ROOT_MODEL_INDEX,
                })?;

        let mut parent_locations = HashMap::from([(
            AggregateModelIndex::ROOT_MODEL_INDEX,
            CalcAnimUnitIndex::ROOT_UNIT_INDEX,
        )]);

        for (
            AggregateUnit {
                model,
                offset,
                skeleton_attachment_point: attachment_point,
                parent_unit: parent_model_unit,
                ..
            },
            index,
        ) in self
            .children
            .iter()
            .zip(indices)
            .filter_map(|(opt, index)| opt.as_ref().map(|unit| (unit, index)))
        {
            let animation = animations.get(&index).copied();
            let parent_unit_index = parent_locations
                .get(parent_model_unit)
                .expect("parent comes before child in `self.children`, per the push_unit method");
            let destination = anim_builder
                .push(
                    *parent_unit_index,
                    *attachment_point,
                    *offset,
                    model.skeleton(),
                    animation,
                )
                .map_err(|error| match error {
                    PushAnimationError::SignaturesDoNotMatch(error) => {
                        CalculateAnimationError::BadAnimationInput {
                            error,
                            parent: *parent_model_unit,
                        }
                    }
                    _ => unreachable!("should be well formed animation tree"),
                })?;
            parent_locations.insert(index, destination);
        }

        let anim = if let Some((unit_index, bone_index, pose)) = fix_anim {
            let unit_index = *parent_locations
                .get(&unit_index)
                .ok_or(CalculateAnimationError::FixAnimationUnitDoesNotExist)?;
            anim_builder
                .build_fixing_bone(unit_index, bone_index, pose)
                .map_err(|error| match error {
                    BuildFixingBoneError::BoneDoesNotExist => {
                        CalculateAnimationError::FixAnimationBoneDoesNotExist
                    }
                    BuildFixingBoneError::UnitDoesNotExist => {
                        unreachable!("checked above when getting CalcAnimUnitIndex")
                    }
                })?
        } else {
            anim_builder.build()
        };

        Ok(anim)
    }

    pub(crate) fn zip_calculated_animation_opt<'a, 'b>(
        &'a self,
        aggr: &'b AggregateCalcAnim,
    ) -> impl Iterator<
        Item = Option<(
            &'a M,
            Option<&'a M::BoundingBox<T>>,
            &'b CalculatedAnimation,
        )>,
    > {
        let mut aggr_iter = aggr.vec.iter();
        iter::once(Some((
            &self.base_model,
            self.base_bounding_box.as_ref(),
            aggr_iter
                .next()
                .expect("CalcAnim must have been created from this entity"),
        )))
        .chain(self.children.iter().map(move |child| {
            child.as_ref().map(|aggr| {
                (
                    &aggr.model,
                    aggr.bounding_box.as_ref(),
                    aggr_iter
                        .next()
                        .expect("CalcAnim must have been created from this entity"),
                )
            })
        }))
    }

    pub(crate) fn zip_calculated_animation<'a, 'b>(
        &'a self,
        aggr: &'b AggregateCalcAnim,
    ) -> impl Iterator<
        Item = (
            &'a M,
            Option<&'a M::BoundingBox<T>>,
            &'b CalculatedAnimation,
        ),
    > {
        self.zip_calculated_animation_opt(aggr).flatten()
    }

    pub(crate) fn zip_calculated_animation_get_index_opt<'a, 'b>(
        &'a self,
        aggr: &'b AggregateCalcAnim,
    ) -> impl Iterator<
        Item = Option<(
            AggregateModelIndex,
            &'a M,
            Option<&'a M::BoundingBox<T>>,
            &'b CalculatedAnimation,
        )>,
    > {
        self.zip_calculated_animation_opt(aggr)
            .enumerate()
            .map(|(index, opt)| opt.map(|(x, y, z)| (AggregateModelIndex(index), x, y, z)))
    }

    pub(crate) fn zip_calculated_animation_get_index<'a, 'b>(
        &'a self,
        aggr: &'b AggregateCalcAnim,
    ) -> impl Iterator<
        Item = (
            AggregateModelIndex,
            &'a M,
            Option<&'a M::BoundingBox<T>>,
            &'b CalculatedAnimation,
        ),
    > {
        self.zip_calculated_animation_get_index_opt(aggr).flatten()
    }
}

// impl<T, M: EntityInner> AggregateEntity<T, M>
// where
// M::Vertex: CubicVertex,
// {
// pub fn detect_collision<F, O>(
// lhs: &Self,
// lhs_calculated_animation: &AggregateCalcAnim,
// lhs_alter_meta: F,
//
// rhs: &AggregateEntity<>,
// rhs_calculated_animation: &AggregateCalcAnim,
// rhs_alter_meta: O,
// ) -> bool
// where
// F: Fn(AggregateModelIndex, <M::BoundingBox<T> as
// BoundingBoxTrait>::FuncInputs, &mut <M::BoundingBox<T> as
// BoundingBoxTrait>::Inner), O: Fn(AggregateModelIndex, SkeletonBoneIndex, &mut
// T), {
// for (lhs_index, _, lhs_bbox, lhs_anim) in
// lhs.zip_calculated_animation_get_index(lhs_calculated_animation) {
// for (rhs_index, _, rhs_bbox, rhs_anim) in
// rhs.zip_calculated_animation_get_index(rhs_calculated_animation) {
// let Some((lhs, rhs)) = lhs_bbox.zip(rhs_bbox) else {return false};
// lhs.intersects_bounding_box(transform, self_alter_meta, target,
// target_transform, target_alter_meta) }
// }
//
//
// todo!()
// }
// }

pub enum DrawError<'a, M: EntityInner + 'a>
where
    M::Vertex: CubicVertex,
{
    ModelDraw(M::DrawError<'a>),
}

impl<'a, M: EntityInner + 'a> Debug for DrawError<'a, M>
where
    M::Vertex: CubicVertex,
    M::DrawError<'a>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DrawError::ModelDraw(err) => Debug::fmt(err, f),
        }
    }
}

impl<'a, M: EntityInner + 'a> Display for DrawError<'a, M>
where
    M::Vertex: CubicVertex,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl<'a, M: EntityInner + 'a> std::error::Error for DrawError<'a, M> where M::Vertex: CubicVertex {}

impl<T, M: EntityInner> AggregateEntity<T, M>
where
    M::Vertex: CubicVertex,
{
    pub(crate) fn draw<'a, 'b, const MAX: usize, const OUT: usize, F: FramebufferWithDepth<OUT>>(
        &'a self,
        active_sp: &mut ActiveShaderProgram<'b, '_, 'a, M, ListLights<MAX>, F::Tex, OUT>,
        active_fb: &mut ActiveFramebuffer<'_, '_, OUT, F>,
        calculated_animation: AggregateCalcAnim,
    ) -> Result<(), DrawError<'b, M>> {
        for (model, _, calc) in self.zip_calculated_animation(&calculated_animation) {
            model
                .draw(active_sp, active_fb, calc)
                .map_err(|err| DrawError::<M>::ModelDraw(err))?;
        }

        Ok(())
    }
}

impl<T, M: EntityInner> AggregateEntity<T, M>
where
    <M as Model>::Vertex: CubicVertex,
{
    pub fn push<E: FromToAggregateEntity<T, Model = M>>(
        &mut self,
        entity: E,
        offset: Pose,
        parent_unit: AggregateModelIndex,
        skeleton_attachment: SkeletonBoneIndex,
    ) -> Result<AggregateModelIndex, PushModelError> {
        self.check_parent_and_bone(skeleton_attachment, parent_unit)?;

        let (model, bounding_box) = entity.to_pair_model_bbox();
        let unit = AggregateUnit {
            model,
            bounding_box,
            offset,
            skeleton_attachment_point: skeleton_attachment,
            parent_unit,
            children: Vec::new(),
        };

        Ok(self.push_aggregate_unit(unit))
    }
}

// impl<T> AggregateEntity<T, SkeletonCubic> {
// pub fn push_skeleton_entity(
// &mut self,
// entity: SkeletonEntity<T>,
// offset: Pose,
// parent_unit: AggregateModelIndex,
// skeketon_attachment: SkeletonBoneIndex,
// ) -> Result<AggregateModelIndex, PushModelError> {
// self.check_parent_and_bone(skeketon_attachment, parent_unit)?;
//
// let SkeletonEntity {
// model,
// bounding_box,
// ..
// } = entity;
//
// let unit = AggregateUnit {
// model,
// bounding_box,
// skeleton_attachment_point: skeketon_attachment,
// offset,
// parent_unit,
// children: Vec::new(),
// };
//
// Ok(self.push_aggregate_unit(unit))
// }
// }
//
// impl<T> AggregateEntity<T, SimpleCubic> {
// pub fn push_simple_entity(
// &mut self,
// entity: SimpleEntity<T>,
// offset: Pose,
// parent_unit: AggregateModelIndex,
// skeleton_attachment: SkeletonBoneIndex,
// ) -> Result<AggregateModelIndex, PushModelError> {
// self.check_parent_and_bone(skeleton_attachment, parent_unit)?;
//
// let SimpleEntity {
// model,
// bounding_box,
// ..
// } = entity;
//
// let unit = AggregateUnit {
// model,
// bounding_box,
// skeleton_attachment_point: skeleton_attachment,
// offset,
// parent_unit,
// children: Vec::new(),
// };
//
// Ok(self.push_aggregate_unit(unit))
// }
// }
//
// impl<T> AggregateEntity<T, SkinningCubic> {
// pub fn push_skinning_entity(
// &mut self,
// entity: SkinningEntity<T>,
// offset: Pose,
// parent_unit: AggregateModelIndex,
// skeleton_attachment: SkeletonBoneIndex,
// ) -> Result<AggregateModelIndex, PushModelError> {
// self.check_parent_and_bone(skeleton_attachment, parent_unit)?;
//
// let SkinningEntity {
// model,
// bounding_box,
// ..
// } = entity;
//
// let unit = AggregateUnit {
// model,
// bounding_box,
// skeleton_attachment_point: skeleton_attachment,
// offset,
// parent_unit,
// children: Vec::new(),
// };
//
// Ok(self.push_aggregate_unit(unit))
// }
// }

impl<TThis, M: EntityInner> Collision for AggregateEntity<TThis, M>
where
    M::Vertex: CubicVertex,
    for<'a> <M::BoundingBox<TThis> as Collision>::Transform<'a>:
        From<(&'a Skeleton, &'a CalculatedAnimation)>,
{
    type FuncInputs = (
        AggregateModelIndex,
        <M::BoundingBox<TThis> as Collision>::FuncInputs,
    );
    type Inner = <M::BoundingBox<TThis> as Collision>::Inner;
    type Transform<'a> = AggregateCalcAnim;

    fn collides<
        T: Collision,
        F: FnMut(Self::FuncInputs, &mut Self::Inner),
        O: FnMut(T::FuncInputs, &mut T::Inner),
    >(
        &self,
        transform: &Self::Transform<'_>,
        mut self_alter_meta: F,
        target: &T,
        target_transform: &T::Transform<'_>,
        mut target_alter_meta: O,
    ) -> bool {
        self.zip_calculated_animation_get_index(transform)
            .flat_map(|(x, y, opt_z, t)| opt_z.map(|z| (x, y, z, t)))
            .any(|(index, model, bbox, animation)| {
                let transform = (model.skeleton(), animation);
                let transform = transform.into();

                bbox.collides(
                    &transform,
                    |x, y| self_alter_meta((index, x), y),
                    target,
                    target_transform,
                    &mut target_alter_meta,
                )
            })
    }
}
