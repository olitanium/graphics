use core::time::Duration;
use std::collections::HashSet;

use engine::Key;
use engine::geometry::Pose;
use engine::linear_algebra::{UnitVector, Vector};
use engine::modelling::cubic::CalculatedAnimation;

use crate::state::State;

impl State {
    pub fn physics(
        &mut self,
        frame_time: Duration,
        keyboard: &HashSet<Key>,
        elapsed: Duration,
        player_animation: &CalculatedAnimation,
        ground_animation: &CalculatedAnimation,
    ) {
        let float_time = elapsed.as_secs_f32();
        let new_sun_pos = Vector::new([-float_time.cos().sin(), 1.0, -float_time.cos().cos()]);

        self.sun.inner.physics.pose =
            Pose::new_from_to_up(new_sun_pos, self.player.root().translation(), UnitVector::Y)
                .expect("sun never below player");
        self.sun.light.set_position(new_sun_pos);
        self.sun_duplicate_light.set_position(new_sun_pos);

        self.player.physics(
            &self.game_camera,
            keyboard,
            player_animation,
            &mut self.origin_slope,
            ground_animation,
            frame_time.as_secs_f32(),
        );
    }
}
