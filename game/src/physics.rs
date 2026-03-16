use engine::linear_algebra::Vector;
use engine::modelling::Collision;
use engine::modelling::cubic::geometry::AggregateCalcAnim;

use crate::grappling_hook::UpdateInput;
use crate::morgan::Morgan;
use crate::state::State;

impl State {
    pub fn physics(&mut self, frame_time: f32) -> AggregateCalcAnim {
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

        let mut update_input: UpdateInput<'_, Morgan, Morgan, Morgan> = if self.toggle_b {
            let morgan = self.morgan.take();

            let opt_morgan = morgan.map(|morg| {
                let pose = morg.inner.pose;
                (morg, pose)
            });
            UpdateInput {
                grapple_points: vec![],
                carryable_objects: vec![opt_morgan],
                other_entities: vec![],
            }
        } else {
            let grapple_points = if let Some(morg) = self.morgan.as_ref() {
                let pose = morg.inner.pose;
                vec![(morg, pose)]
            } else {
                vec![]
            };

            UpdateInput {
                grapple_points,
                carryable_objects: vec![],
                other_entities: vec![],
            }
        };

        let grappling_hook_calc_anim = self.grappling_hook.update(frame_time, &mut update_input);

        if self.toggle_b && self.morgan.is_none() {
            self.morgan = update_input
                .carryable_objects
                .pop()
                .flatten()
                .map(|(morgan, _)| morgan)
                .take();
        }

        grappling_hook_calc_anim
    }
}
