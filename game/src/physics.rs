use engine::Transform;
use engine::entity::SkeletonEntity;
use engine::linear_algebra::Vector;
use engine::modelling::Collision;
use engine::modelling::cubic::SkeletonBoneIndex;
use engine::modelling::cubic::geometry::AggregateCalcAnim;

use crate::state::State;

impl State {
    pub fn physics(&mut self, _frame_time: f32) -> AggregateCalcAnim {
        let time = self.time;

        let light_pos = Vector::new([
            5.0 * (time / 10.0).sin(),
            5.0 * (time / 10.0).sin(),
            5.0 * (time / 10.0).cos(),
        ]);

        if let Some(light) = self.light_group.point.get_mut(0) {
            light.light.position = light_pos;
        }

        if let Some(light) = self.light_group.spot.get_mut(0) {
            light.light.direction = self.camera.direction(&());
            light.light.position = self.camera.position(&());
        }

        let mut grappling_hook_calc_anim = self
            .grappling_hook
            .calculate_animation(None, time - self.reset_time);
        if let Some(morgan) = self.morgan.take() {
            let mut was_collision = false;
            self.grappling_hook.entity.collides(
                &grappling_hook_calc_anim,
                |(aggregate_id, _), _| {
                    if aggregate_id == self.grappling_hook.hook {
                        was_collision = true
                    }
                },
                &morgan,
                &morgan.pose,
                |_, _| (),
            );

            if was_collision {
                let hook_pose = grappling_hook_calc_anim
                    .get(self.grappling_hook.hook)
                    .unwrap()
                    .root();
                let morgan_pose = morgan.pose;
                let offset = morgan_pose.transform(&hook_pose.inverse());

                let dest = self
                    .grappling_hook
                    .entity
                    .push(
                        SkeletonEntity::from(morgan),
                        offset,
                        self.grappling_hook.hook,
                        SkeletonBoneIndex::SKELETON_ROOT_BONE,
                    )
                    .unwrap();
                self.grappling_hook.attachment = Some(dest);
                grappling_hook_calc_anim = self
                    .grappling_hook
                    .calculate_animation(None, time - self.reset_time)
            } else {
                self.morgan = Some(morgan)
            }
        }

        grappling_hook_calc_anim
    }
}
