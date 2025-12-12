use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::{iter, mem};

use graphics::ActiveShaderProgram;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Vector;
use utils::error_boilerplate;

use crate::SimpleEntity;
use crate::entity::{EntityInner, SkeletonEntity, SkinningEntity};
use crate::modelling::cubic::geometry::{
    BuildFixingBoneError,
    CalcAnimUnitIndex,
    CalculatedAnimation,
    Pose,
    PushAnimationError,
    SignaturesDoNotMatchError,
    SkeletalAnimation,
};
use crate::modelling::cubic::lighting::simple::ListLights;
use crate::modelling::cubic::{SkeletonBoneIndex, SkeletonCubic};
use crate::modelling::{CubicVertex, SimpleCubic, SkeletonBb};

mod group;
pub use group::Group as AggregateGroup;

mod error;
pub use error::PushModelError;

pub struct AggregateEntity<T, M: EntityInner>
where
    M::Vertex: CubicVertex,
{
    base_model: M,
    base_bounding_box: Option<M::BoundingBox<T>>,
    base_pose: Pose,
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
        animations: &HashMap<AggregateModelIndex, (&'a SkeletalAnimation, f32)>,
    ) -> Result<Vec<CalculatedAnimation>, CalculateAnimationError> {
        let indices = (1..).map(AggregateModelIndex);
        let first_animation = animations
            .get(&AggregateModelIndex::ROOT_MODEL_INDEX)
            .copied();

        let mut anim_builder = CalculatedAnimation::builder(
            self.base_pose,
            self.base_model.skeleton(),
            first_animation,
        )
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
                .expect("parent comes before child in `self.children`");
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
}

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
        calculated_animation: Vec<CalculatedAnimation>,
    ) -> Result<(), DrawError<'b, M>> {
        for (model, calc) in iter::once(&self.base_model)
            .chain(
                self.children
                    .iter()
                    .filter_map(|unit| unit.as_ref().map(|unit| &unit.model)),
            )
            .zip(calculated_animation)
        {
            model
                .draw(active_sp, active_fb, &calc)
                .map_err(|err| DrawError::<M>::ModelDraw(err))?;
        }

        Ok(())
    }
}

impl<T> AggregateEntity<T, SimpleCubic> {
    pub fn push_skeleton_entity(
        &mut self,
        entity: SkeletonEntity<T>,
        offset: Pose,
        parent_unit: AggregateModelIndex,
        skeketon_attachment: SkeletonBoneIndex,
    ) -> Result<AggregateModelIndex, PushModelError> {
        self.check_parent_and_bone(skeketon_attachment, parent_unit)?;

        let SkeletonEntity {
            model,
            bounding_box,
            ..
        } = entity;

        let unit = AggregateUnit {
            model,
            bounding_box,
            skeleton_attachment_point: skeketon_attachment,
            offset,
            parent_unit,
            children: Vec::new(),
        };

        Ok(self.push_aggregate_unit(unit))
    }
}

impl<T> AggregateEntity<T, SimpleCubic> {
    pub fn push_simple_entity(
        &mut self,
        entity: SimpleEntity<T>,
        offset: Pose,
        parent_unit: AggregateModelIndex,
        skeleton_attachment: SkeletonBoneIndex,
    ) -> Result<AggregateModelIndex, PushModelError> {
        self.check_parent_and_bone(skeleton_attachment, parent_unit)?;

        let SimpleEntity {
            model,
            bounding_box,
            ..
        } = entity;
        let bounding_box_new = bounding_box.map(|bbox| SkeletonBb::from_single(bbox));

        let unit = AggregateUnit {
            model,
            bounding_box: bounding_box_new,
            skeleton_attachment_point: skeleton_attachment,
            offset,
            parent_unit,
            children: Vec::new(),
        };

        Ok(self.push_aggregate_unit(unit))
    }
}

impl<T> AggregateEntity<T, SkeletonCubic> {
    pub fn push_skinning_entity(
        &mut self,
        entity: SkinningEntity<T>,
        offset: Pose,
        parent_unit: AggregateModelIndex,
        skeleton_attachment: SkeletonBoneIndex,
    ) -> Result<AggregateModelIndex, PushModelError> {
        self.check_parent_and_bone(skeleton_attachment, parent_unit)?;

        let SkinningEntity {
            model,
            bounding_box,
            ..
        } = entity;

        let unit = AggregateUnit {
            model,
            bounding_box,
            skeleton_attachment_point: skeleton_attachment,
            offset,
            parent_unit,
            children: Vec::new(),
        };

        Ok(self.push_aggregate_unit(unit))
    }
}
