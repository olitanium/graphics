use std::iter;

use graphics::ActiveShaderProgram;
use graphics::framebuffer::ActiveFramebuffer;
use graphics::framebuffer::traits::FramebufferWithDepth;

use crate::EngResult;
use crate::modelling::SkeletalVertex;
use crate::modelling::cubic::SkeletonCubic;
use crate::modelling::cubic::geometry::{CalculatedAnimation, Pose, SkeletalAnimation};
use crate::modelling::cubic::lighting::simple::ListLights;

mod group;
pub use group::Group as AggregateGroup;

#[derive(Debug)]
pub struct AggregateCubic {
    base_model: SkeletonCubic,
    children: Vec<(
        SkeletonCubic,
        Pose,  // offset from base bone
        usize, // bone index
    )>,
}

impl AggregateCubic {
    pub fn new(base_model: SkeletonCubic) -> Self {
        Self {
            base_model,
            children: Vec::new(),
        }
    }

    pub fn push(&mut self, other_model: SkeletonCubic, offset: Pose, attachment_point: usize) {
        self.children.push((other_model, offset, attachment_point));
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
            SkeletalVertex,
            (SkeletonCubic, ListLights<MAX>),
            F::Tex,
            OUT,
        >,
        active_fb: &mut ActiveFramebuffer<'_, '_, OUT, F>,
        pose: Pose,
        animation: Option<&SkeletalAnimation>,
        time: f32,
        animation2: Option<&SkeletalAnimation>,
        time2: f32,
    ) -> EngResult<(), NEVER> {
        let mut anim_builder =
            CalculatedAnimation::builder(pose, &self.base_model.skeleton, animation, time)?;
        for (model, offset, attachment_point) in &self.children {
            anim_builder.push(
                0,
                *attachment_point,
                *offset,
                &model.skeleton,
                animation2,
                time2,
            )?;
        }

        let anim = anim_builder.build();

        for (model, calc) in iter::once(&self.base_model)
            .chain(self.children.iter().map(|(model, ..)| model))
            .zip(anim)
        {
            model.draw(active_sp, active_fb, calc)?;
        }

        Ok(())
    }
}
