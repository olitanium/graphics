use std::fmt::Debug;
use std::{iter, mem};

use graphics::ActiveShaderProgram;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Vector;

use crate::entity::{EntityInner, SkeletonEntity, SkinningEntity};
use crate::modelling::cubic::geometry::{CalculatedAnimation, Pose, SkeletalAnimation};
use crate::modelling::cubic::lighting::simple::ListLights;
use crate::modelling::cubic::{SkeletonBoneIndex, SkeletonCubic};
use crate::modelling::{SimpleCubic, SkeletonBb};
use crate::{EngResult, SimpleEntity};

mod group;
pub use group::Group as AggregateGroup;

pub struct AggregateEntity<T, M: EntityInner> {
    base: AggregateUnit<T, M>,
    children: Vec<Option<AggregateUnit<T, M>>>,
}

impl<T: Debug, M: EntityInner + Debug> Debug for AggregateEntity<T, M>
where
    M::BoundingBox<T>: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(AggregateEntity))
            .field("base", &self.base)
            .field("children", &self.children)
            .finish()
    }
}

#[derive(Debug)]
struct AggregateUnit<T, M: EntityInner> {
    model: M,
    bounding_box: Option<M::BoundingBox<T>>,
    offset: Pose,
    skeleton_attachment_point: SkeletonBoneIndex,
    parent_unit: AggregateModelIndex,
    children: Vec<AggregateModelIndex>,
}

#[derive(Debug, Clone, Copy)]
pub struct AggregateModelIndex(pub(crate) usize);
impl AggregateModelIndex {
    pub const ROOT_MODEL_INDEX: Self = Self(0);
}

impl<T, M: EntityInner> AggregateEntity<T, M> {
    pub fn new<F: FnMut(&[(Vector<3>, [M::Vertex; 3])]) -> T>(
        base_model: M,
        depth: usize,
        init: F,
    ) -> Self {
        let bounding_box = base_model.make_bounding_box(depth, init);

        let base = AggregateUnit {
            model: base_model,
            bounding_box,
            offset: Pose::IDENTITY,
            skeleton_attachment_point: SkeletonBoneIndex::SKELETON_ROOT_BONE,
            parent_unit: AggregateModelIndex::ROOT_MODEL_INDEX,
            children: Vec::new(),
        };

        Self {
            base,
            children: Vec::new(),
        }
    }

    pub fn push_model<F: FnMut(&[(Vector<3>, [M::Vertex; 3])]) -> T>(
        &mut self,
        other_model: M,
        offset: Pose,
        attachment_point: SkeletonBoneIndex,
        parent_unit: AggregateModelIndex,

        bounding_box_depth: usize,
        bounding_box_init: F,
    ) -> Option<AggregateModelIndex> {
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

        Some(child_at)
    }

    fn check_parent_and_bone(
        &self,
        attachment_point: SkeletonBoneIndex,
        parent_unit: AggregateModelIndex,
    ) -> Option<()> {
        self.get(parent_unit)?
            .model
            .skeleton()
            .get(attachment_point)
            .is_bone()
            .then_some(())
    }

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

        let parent_mut = self.get_mut(parent_unit).expect("asserted before");
        parent_mut.children.push(child_at);

        child_at
    }

    pub fn pop(&mut self, index: AggregateModelIndex) {
        if index.0 == AggregateModelIndex::ROOT_MODEL_INDEX.0 {
            return;
        }
        if let Some(Some(unit)) = self.children.get_mut(index.0 - 1) {
            for unit_index in mem::take(&mut unit.children) {
                self.pop(unit_index);
            }
        }
        if let Some(opt_empty_unit) = self.children.get_mut(index.0 - 1) {
            *opt_empty_unit = None;
        }
    }

    fn get(&self, index: AggregateModelIndex) -> Option<&AggregateUnit<T, M>> {
        if index.0 == AggregateModelIndex::ROOT_MODEL_INDEX.0 {
            Some(&self.base)
        } else {
            self.children.get(index.0 - 1).and_then(|opt| opt.as_ref())
        }
    }

    fn get_mut(&mut self, index: AggregateModelIndex) -> Option<&mut AggregateUnit<T, M>> {
        if index.0 == AggregateModelIndex::ROOT_MODEL_INDEX.0 {
            Some(&mut self.base)
        } else {
            self.children
                .get_mut(index.0 - 1)
                .and_then(|opt| opt.as_mut())
        }
    }

    fn iter_units(&self) -> impl Iterator<Item = &AggregateUnit<T, M>> {
        iter::once(&self.base).chain(self.children.iter().flatten())
    }

    pub(crate) fn draw<
        'a,
        NEVER,
        const MAX: usize,
        const OUT: usize,
        F: FramebufferWithDepth<OUT>,
    >(
        &'a self,
        active_sp: &mut ActiveShaderProgram<
            '_,
            '_,
            'a,
            M::Vertex,
            (M, ListLights<MAX>),
            F::Tex,
            OUT,
        >,
        active_fb: &mut ActiveFramebuffer<'_, '_, OUT, F>,
        animations: Vec<Option<(&SkeletalAnimation, f32)>>,
    ) -> EngResult<(), NEVER> {
        let mut animations = animations.into_iter().chain(iter::repeat(None));
        let first_animation = animations.next().flatten();

        let mut anim_builder = CalculatedAnimation::builder(
            self.base.offset,
            self.base.model.skeleton(),
            first_animation,
        )?;
        for (
            AggregateUnit {
                model,
                offset,
                skeleton_attachment_point: attachment_point,
                parent_unit,
                ..
            },
            animation,
        ) in self.children.iter().flatten().zip(animations)
        {
            anim_builder.push(
                *parent_unit,
                *attachment_point,
                *offset,
                model.skeleton(),
                animation,
            )?;
        }

        let anim = anim_builder.build();

        for (model, calc) in iter::once(&self.base)
            .chain(self.children.iter().flatten())
            .map(|AggregateUnit { model, .. }| model)
            .zip(anim)
        {
            model.draw(active_sp, active_fb, &calc)?;
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
    ) -> Option<AggregateModelIndex> {
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

        Some(self.push_aggregate_unit(unit))
    }
}

impl<T> AggregateEntity<T, SimpleCubic> {
    pub fn push_simple_entity(
        &mut self,
        entity: SimpleEntity<T>,
        offset: Pose,
        parent_unit: AggregateModelIndex,
        skeleton_attachment: SkeletonBoneIndex,
    ) -> Option<AggregateModelIndex> {
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

        Some(self.push_aggregate_unit(unit))
    }
}
impl<T> AggregateEntity<T, SkeletonCubic> {
    pub fn push_skinning_entity(
        &mut self,
        entity: SkinningEntity<T>,
        offset: Pose,
        parent_unit: AggregateModelIndex,
        skeleton_attachment: SkeletonBoneIndex,
    ) -> Option<AggregateModelIndex> {
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

        Some(self.push_aggregate_unit(unit))
    }
}
