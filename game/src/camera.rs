use std::collections::HashSet;
use std::time::Duration;

use engine::Key;
use engine::linear_algebra::Vector;
use engine::modelling::cubic;

pub struct GameCamera {
    pub move_lock: bool,
    pub camera: cubic::Camera,
    pub sensitivity: f32,
    pub speed: f32,
}

impl GameCamera {
    pub fn controls(
        &mut self,
        keyboard: &HashSet<Key>,
        _typing_string: &mut String,
        mouse_delta: Vector<2>,
        frame_time: f32,
    ) {
        if self.move_lock {
            let distance = frame_time * self.speed;

            if keyboard.contains(&Key::Right) | keyboard.contains(&Key::D) {
                self.camera.pose.move_left(-distance);
            }
            if keyboard.contains(&Key::Left) | keyboard.contains(&Key::A) {
                self.camera.pose.move_left(distance);
            }
            if keyboard.contains(&Key::Up) | keyboard.contains(&Key::W) {
                self.camera.pose.move_forward(distance);
            }
            if keyboard.contains(&Key::Down) | keyboard.contains(&Key::S) {
                self.camera.pose.move_forward(-distance);
            }
            if keyboard.contains(&Key::Space) {
                self.camera.pose.move_up(distance);
            }
            if keyboard.contains(&Key::LeftShift) {
                self.camera.pose.move_up(-distance);
            }
            self.camera
                .pose
                .look_left(-mouse_delta[0] * self.sensitivity);
            self.camera.pose.look_up(mouse_delta[1] * self.sensitivity);
        }
    }
}
