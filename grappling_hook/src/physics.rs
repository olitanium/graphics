use engine::linear_algebra::Vector;
use engine::modelling::cubic::AggregateCalcAnim;

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

        self.point_light.light.position = light_pos;

        self.spotlight.light.direction = self.camera.direction(&());
        self.spotlight.light.position = self.camera.position(&());

        // self.morgan = None;

        let mut update_input: UpdateInput<'_, Morgan, Morgan, Morgan> = if self.toggle_b {
            let morgan = self.morgan.take();

            let opt_morgan = morgan.map(|morg| (morg, (), ()));
            UpdateInput {
                grapple_points: vec![],
                carryable_objects: vec![opt_morgan],
                other_entities: vec![],
            }
        } else {
            let grapple_points = if let Some(morg) = self.morgan.as_ref() {
                vec![(morg, (), ())]
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
                .map(|(morgan, ..)| morgan)
                .take();
        }

        grappling_hook_calc_anim
    }
}
