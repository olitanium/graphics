use engine::linear_algebra::{UnitVector, Vector};

use crate::state::State;

impl State {
    pub fn physics(&mut self, frame_time: f32) {
        let game_start = *self.game_start.get_or_insert(self.time);
        let time = self.time - game_start;

        let light_pos = Vector::new([
            5.0 * (time / 10.0).sin(),
            5.0 * (time / 10.0).sin(),
            5.0 * (time / 10.0).cos(),
        ]);

        if let Some(light) = self.light_group.point.get_mut(0) {
            light.light.position = light_pos;
        }

        // self.imported_other.update(frame_time);
        // self.imported.update(frame_time);

        // self.imported.pose = Pose::new_from_orientation_translation(
        // Orientation::new_from_to(Vector::ZERO, light_pos, UnitVector::Y),
        // Vector::ZERO,
        // );

        if let Some(light) = self.light_group.spot.get_mut(0) {
            light.light.direction = self.camera.direction(&());
            light.light.position = self.camera.position(&());
        }
    }
}
