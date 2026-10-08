use engine::linear_algebra::vector;
use engine::modelling::cubic::AggregateCalcAnim;

use crate::bear::Bear;
use crate::grappling_hook::UpdateInput;
use crate::state::State;

impl State {
    pub fn physics(&mut self, frame_time: f32) -> AggregateCalcAnim {
        let time = self.time;

        let light_pos = vector![
            5.0 * (time / 10.0).sin(),
            5.0 * (time / 10.0).sin(),
            5.0 * (time / 10.0).cos(),
        ];

        self.point_light.light.position = light_pos;

        self.spotlight.light.direction = self.camera.direction(&());
        self.spotlight.light.position = self.camera.position(&());

        let mut update_input: UpdateInput<'_, Bear, Bear, Bear> = if self.toggle_b {
            let bear = self.bear.take();

            let opt_bear = bear.map(|morg| (morg, (), ()));
            UpdateInput {
                grapple_points: vec![],
                carryable_objects: vec![opt_bear],
                other_entities: vec![],
            }
        } else {
            // let grapple_points = if let Some(bear) = self.bear.as_ref() {
            // vec![(bear, (), ())]
            // } else {
            // vec![]
            // };
            //
            // UpdateInput {
            // grapple_points,
            // carryable_objects: vec![],
            // other_entities: vec![],
            // }

            UpdateInput {
                grapple_points: vec![],
                carryable_objects: vec![],
                other_entities: vec![],
            }
        };

        let grappling_hook_calc_anim = self.grappling_hook.update(frame_time, &mut update_input);

        if self.toggle_b && self.bear.is_none() {
            self.bear = update_input
                .carryable_objects
                .pop()
                .flatten()
                .map(|(bear, ..)| bear)
        }

        grappling_hook_calc_anim
    }
}
