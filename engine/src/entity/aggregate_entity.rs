use std::fmt::Debug;
use std::iter;

use graphics::ActiveShaderProgram;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Vector;

use crate::EngResult;
use crate::entity::EntityInner;
use crate::modelling::cubic::SkeletonBoneIndex;
use crate::modelling::cubic::geometry::{CalculatedAnimation, Pose, SkeletalAnimation};
use crate::modelling::cubic::lighting::simple::ListLights;

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
        };
        Self {
            base,
            children: Vec::new(),
        }
    }

    pub fn push<F: FnMut(&[(Vector<3>, [M::Vertex; 3])]) -> T>(
        &mut self,
        other_model: M,
        offset: Pose,
        attachment_point: SkeletonBoneIndex,
        parent_unit: AggregateModelIndex,

        bounding_box_depth: usize,
        bounding_box_init: F,
    ) -> Option<AggregateModelIndex> {
        // check that the parent unit is Some and the bone exists
        let parent = self.get(parent_unit.0)?;
        if !parent.model.skeleton().get(attachment_point).is_bone() {
            return None;
        }

        let bounding_box = other_model.make_bounding_box(bounding_box_depth, bounding_box_init);

        let child = AggregateUnit {
            model: other_model,
            bounding_box,
            offset,
            skeleton_attachment_point: attachment_point,
            parent_unit,
        };
        // get index of the first None

        if let Some((index, child_dest)) = self
            .children
            .iter_mut()
            .enumerate()
            .find(|(_, opt_unit)| opt_unit.is_none())
        {
            *child_dest = Some(child);
            Some(AggregateModelIndex(index + 1))
        } else {
            self.children.push(Some(child));
            let output = self.children.len();
            Some(AggregateModelIndex(output))
        }
    }

    fn get(&self, index: usize) -> Option<&AggregateUnit<T, M>> {
        if index == 0 {
            Some(&self.base)
        } else {
            self.children
                .get(index - 1)
                .and_then(|opt| opt.as_ref())
        }
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
            // TODO: I don't like flatten here.
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
