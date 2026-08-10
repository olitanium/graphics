use std::collections::HashSet;

use engine::Key;
use engine::framebuffer::Framebuffer;
use engine::framebuffer::attachments::WithDepth;
use engine::framebuffer::traits::FramebufferInternals;
use engine::linear_algebra::{UnitVector, Vector};
use engine::modelling::cubic::{self, Camera, CameraBuilder, ManualCameraPose};

pub struct GameCamera {
    pub move_lock: bool,
    pub camera: cubic::Camera,
    pub sensitivity: f32,
    pub speed: f32,
}

impl GameCamera {
    pub fn new(hdr_fb: &Framebuffer<3, WithDepth>) -> Self {
        const START_LOC: Vector<3> = Vector::new([0.0, 0.5, 1.0]);

        let camera = CameraBuilder::new()
            .pose(ManualCameraPose::new_fixed_up_from_to(
                START_LOC,
                Vector::ZERO,
                UnitVector::Y,
            ))
            .perspective(90.0, hdr_fb.aspect_ratio(), 0.01, 100.0)
            .build();
        let sensitivity = 0.01;

        Self {
            move_lock: false,
            camera,
            sensitivity,
            speed: 0.3,
        }
    }

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
