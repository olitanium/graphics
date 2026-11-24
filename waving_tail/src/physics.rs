use std::collections::HashMap;

use engine::entity::AggregateModelIndex;
use engine::modelling::cubic::AggregateCalcAnim;

use crate::state::State;

impl State {
    pub fn physics(&mut self, time: f32) -> AggregateCalcAnim {
        let animations = HashMap::from([
            (
                &AggregateModelIndex::ROOT_MODEL_INDEX,
                (&self.tail_animation, time),
            ),
            (&self.other_tail_index, (&self.tail_animation, time)),
        ]);
        let tail_animation = self.tail.calculate_animation(None, animations);

        tail_animation.unwrap()
    }
}
