use std::fmt::Debug;
use std::iter;

use graphics::ActiveShaderProgram;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;
use graphics::linear_algebra::Vector;

use crate::EngResult;
use crate::entity::EntityInner;
use crate::modelling::cubic::geometry::{CalculatedAnimation, Pose, SkeletalAnimation};
use crate::modelling::cubic::lighting::simple::ListLights;

mod group;
pub use group::Group as AggregateGroup;

#[derive(Debug)]
pub struct AggregateEntity<T: Debug, M: EntityInner> {
    base_model: M,
    bounding_box: Option<M::BoundingBox<T>>,
    children: Vec<AggregateUnit<T, M>>,
}

#[derive(Debug)]
struct AggregateUnit<T: Debug, M: EntityInner> {
    model: M,
    bounding_box: Option<M::BoundingBox<T>>,
    offset: Pose,
    attachment_point: usize,
    parent_unit: usize,
}

impl<T: Debug, M: EntityInner> AggregateEntity<T, M> {
    pub fn new<
F: FnMut(&[(Vector<3>, [M::Vertex; 3])]) -> T
    >(base_model: M, depth: usize, init: F) -> Self {
        let bounding_box = base_model.make_bounding_box(depth, init);
        Self {
            base_model,
            bounding_box, 
            children: Vec::new(),
        }
    }

    pub fn push<F: FnMut(&[(Vector<3>, [M::Vertex; 3])]) -> T>(
        &mut self,
        other_model: M,
        offset: Pose,
        attachment_point: usize,
        parent_unit: usize,

        bounding_box_depth: usize,
        bounding_box_init: F,
    ) {
        let bounding_box = other_model.make_bounding_box(bounding_box_depth, bounding_box_init);
        self.children.push(AggregateUnit {
            model: other_model,
            bounding_box,
            offset,
            attachment_point,
            parent_unit,
        });
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
        root_pose: Pose,
        animations: Vec<Option<(&SkeletalAnimation, f32)>>,
    ) -> EngResult<(), NEVER> {
        let mut animations = animations.into_iter().chain(iter::repeat(None));
        let first_animation = animations.next().flatten();
        let mut anim_builder =
            CalculatedAnimation::builder(root_pose, self.base_model.skeleton(), first_animation)?;
        for (
            AggregateUnit {
                model,
                offset,
                attachment_point,
                parent_unit,
                ..
            },
            animation,
        ) in self.children.iter().zip(animations)
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

        for (model, calc) in iter::once(&self.base_model)
            .chain(
                self.children
                    .iter()
                    .map(|AggregateUnit { model, .. }| model),
            )
            .zip(anim)
        {
            model.draw(active_sp, active_fb, &calc)?;
        }

        Ok(())
    }
}
