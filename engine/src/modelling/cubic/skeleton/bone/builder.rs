use std::cell::Cell;
use std::rc::Rc;

use super::Bone;
use crate::builder;
use crate::modelling::cubic::geometry::{Animation, Pose};

#[derive(Debug, Clone)]
pub struct Builder<T> {
    parent: usize,
    inverse_bind_pose: Option<Pose>,
    animations: T,
}

impl Builder<Vec<Animation>> {
    builder!(inverse_bind_pose: Option<Pose>);

    pub fn new(parent: usize) -> Self {
        Self {
            parent,
            inverse_bind_pose: Default::default(),
            animations: vec![],
        }
    }

    pub fn bind_pose(self, bind_pose: Pose) -> Self {
        Self {
            inverse_bind_pose: Some(bind_pose.inverse()),
            ..self
        }
    }

    pub fn push_animation(mut self, animation: Animation) -> Self {
        self.animations.push(animation);
        self
    }

    pub fn all_animations(self, all_animations: Rc<[Animation]>) -> Builder<Rc<[Animation]>> {
        Builder {
            animations: all_animations,
            ..self
        }
    }
}

impl<T: Into<Rc<[Animation]>>> Builder<T> {
    pub fn build(self) -> Bone {
        Bone {
            parent: Some(self.parent),
            inverse_bind_pose: self.inverse_bind_pose.unwrap_or_default(),
            animation: self.animations.into(),
            last_query: Cell::default(),
        }
    }
}
