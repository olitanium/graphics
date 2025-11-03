use engine::linear_algebra::Vector;

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
            //*self.light = Pose::new_from_translation(light_pos);
        }

        // self.imported_other.update(frame_time);
        self.imported.update(frame_time);

        // let self_pose = self.imported.model.root();
        // let other_pose = self.imported_other.model.root();
        // if !self.has_collided
        // && self.imported.elastic_collision_no_rotation(
        // |&mut (ref mut collided, ref mut triangles)| {
        // collided = true;
        // }, /* triangles.iter_mut().for_each(|triangle| {*triangle =
        // triangle.transform_by(&self_pose);})}, */
        // &mut self.imported_other,
        // |&mut (ref mut collided, ref mut triangles)| {
        // collided = true;
        // }, /* triangles.iter_mut().for_each(|triangle| {*triangle =
        // triangle.transform_by(&other_pose);})}, */
        // )
        // {
        // self.has_collided = true;
        // self.imported.velocity = Vector::ZERO;
        // self.imported_other.velocity = Vector::ZERO;
        // self.imported.model.root_mut().
        // push_change(Pose::new_from_orientation(Orientation::new_forward_up(Vector::new([1.
        // 0, 0.0, 0.0]).normalize(), UnitVector::Y)));
        // self.imported_other = Entity::default();
        // };
        if let Some(light) = self.light_group.spot.get_mut(0) {
            light.light.direction = self.camera.direction(&());
            light.light.position = self.camera.position(&());
        }
    }
}
