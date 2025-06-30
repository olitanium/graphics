use std::cell::Cell;
use std::rc::Rc;

use crate::modelling::cubic::geometry::{Animation, Pose, YieldsPose};

mod builder;
pub use builder::Builder;

#[derive(Debug, Clone)]
pub struct Bone {
    pub(crate) parent: Option<usize>,
    pub(crate) inverse_bind_pose: Pose,
    pub(crate) animation: Rc<[Animation]>,
    pub(crate) last_query: Cell<Option<((usize /* Animation index */, f32 /* time */), Pose)>>,
}

impl Bone {
    pub fn builder(parent: usize) -> Builder<Vec<Animation>> {
        Builder::new(parent)
    }

    pub(crate) fn root() -> Self {
        Self {
            parent: None,
            inverse_bind_pose: Pose::default(),
            animation: Rc::default(),
            last_query: Cell::default(),
        }
    }

    pub fn inverse_bind_pose(&self) -> Pose {
        self.inverse_bind_pose
    }

    pub fn set(&mut self, pose: Pose) {
        let mut animation = Animation::new();
        animation
            .push_final_non_repeat(pose)
            .expect("Fresh animation above");

        self.animation = Rc::new([animation]);
    }
}

impl<'a> YieldsPose for &'a Bone {
    type Hint = (
        &'a [Bone], // skeleton, for back-propogation
        usize,      // animation
        f32,        // time
    );

    fn get_pose(&self, hint: Self::Hint) -> Pose {
        let (skeleton, animation_index, time) = hint;

        if let Some(pose) = self
            .last_query
            .get()
            .and_then(|(inner_hint, pose)| (inner_hint == (animation_index, time)).then_some(pose))
        {
            return pose;
        }

        let parent_pose = self
            .parent
            .and_then(|index| skeleton.get(index))
            .map(|parent| parent.get_pose(hint))
            .unwrap_or_default();

        let own_pose = self
            .animation
            .get(animation_index)
            .map(|anim| anim.get_pose(time))
            .unwrap_or_default();

        let total_pose = own_pose.apply_after(parent_pose);

        self.last_query
            .set(Some(((animation_index, time), total_pose)));

        total_pose
    }
}
