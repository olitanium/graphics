use std::collections::HashSet;

use engine::Key;
use engine::{Error, Result};

use crate::state::State;

impl State {
    pub fn controls(
        &mut self,
        mouse_delta: (f32, f32),
        keyboard: HashSet<Key>,
        typing_string: String,
        frame_time: f32,
    ) -> Result<()> {
        if keyboard.contains(&Key::Escape) {
            return Err(Error::Close);
        }
        if keyboard.contains(&Key::W) {
            self.camera.pose.move_forward(self.speed[2] * frame_time);
        }
        if keyboard.contains(&Key::A) {
            self.camera.pose.move_left(self.speed[0] * frame_time);
            // self.camera.move_right(-self.speed[0] * frame_time);
        }
        if keyboard.contains(&Key::D) {
            self.camera.pose.move_left(-self.speed[0] * frame_time)
            // self.camera.move_right(self.speed[0] * frame_time);
        }
        if typing_string.to_lowercase().contains('b') {
            self.do_bloom = !self.do_bloom;
        }
        if keyboard.contains(&Key::S) {
            self.camera.pose.move_forward(-self.speed[2] * frame_time);
        }
        if keyboard.contains(&Key::LeftShift) | keyboard.contains(&Key::RightShift) {
            self.camera.pose.move_up(-self.speed[2] * frame_time);
        }
        if keyboard.contains(&Key::Space) {
            self.camera.pose.move_up(self.speed[2] * frame_time);
        }
        if keyboard.contains(&Key::R) {
            self.exposure -= 1.2 * frame_time;
        }
        if keyboard.contains(&Key::E) {
            self.exposure += 1.2 * frame_time;
        }
        if keyboard.contains(&Key::Right) {
            self.camera
                .pose
                .look_left(-self.sensitivity * frame_time * 500.0);
        }
        if keyboard.contains(&Key::Left) {
            self.camera
                .pose
                .look_left(self.sensitivity * frame_time * 500.0);
        }
        if keyboard.contains(&Key::U) {
            self.camera.radius_out(-0.5 * frame_time);
        }
        if keyboard.contains(&Key::J) {
            self.camera.radius_out(0.5 * frame_time);
        }
        if typing_string.contains('y') {
            self.which_animation = (self.which_animation + 1) % 2
        }
        if keyboard.contains(&Key::Comma) {
            self.camera.pose.roll_ccw(frame_time);
        }
        if keyboard.contains(&Key::Period) {
            self.camera.pose.roll_ccw(-frame_time);
        }

        self.camera
            .pose
            .look_left(-mouse_delta.0 * self.sensitivity);
        self.camera.pose.look_up(mouse_delta.1 * self.sensitivity);

        Ok(())
    }
}
