use std::time::Duration;

use engine::geometry::{Orientation, Pose};
use engine::linear_algebra::{UnitVector, Vector};

use crate::state::State;

impl State {
    pub fn physics(&mut self, _frame_time: Duration, elapsed: Duration) {
        let float_time = elapsed.as_secs_f32();
        let new_sun_pos = Vector::new([-float_time.cos().sin(), 1.0, -float_time.cos().cos()]);

        self.sun.inner.pose = Pose::new_from_orientation_translation(
            Orientation::new_from_to(new_sun_pos, self.player.root().translation(), UnitVector::Y)
                .expect("sun never below player"),
            new_sun_pos,
        );
        self.sun.light.set_position(new_sun_pos);
        self.sun_duplicate_light.set_position(new_sun_pos);
    }
}
