use std::collections::HashSet;

use engine::Key;
use engine::framebuffer::Framebuffer;
use engine::framebuffer::attachments::WithDepth;
use engine::framebuffer::traits::FramebufferInternals as _;
use engine::linear_algebra::{UnitVector, Vector};
use engine::modelling::cubic::{self, CameraBuilder, ManualPoseFixedUp};
use utils::hash_matches;

pub struct GameCamera {
    pub move_lock: bool,
    pub camera: cubic::Camera<ManualPoseFixedUp>,
    pub sensitivity: f32,
    pub speed: f32,
}

impl GameCamera {
    pub fn new(hdr_fb: &Framebuffer<3, WithDepth>) -> Self {
        const START_LOC: Vector<3> = Vector::new([0.0, 0.5, 1.0]);

        let camera = CameraBuilder::new()
            .pose(ManualPoseFixedUp::new_from_to(START_LOC, Vector::ZERO, UnitVector::Y).unwrap())
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
            let mut displacement = Vector::ZERO;

            hash_matches! { keyboard:
                &Key::Right => displacement += -Vector::X,
                &Key::D => displacement += -Vector::X,
                &Key::A => displacement += Vector::X,
                &Key::Left => displacement += Vector::X,
                &Key::W => displacement += Vector::Z,
                &Key::Up => displacement += Vector::Z,
                &Key::S => displacement += -Vector::Z,
                &Key::Down => displacement += -Vector::Z,
                &Key::Space => displacement += Vector::Y,
                &Key::LeftShift => displacement += -Vector::Y,
            }

            if let Some(displacement) = displacement.try_normalize() {
                self.camera
                    .pose
                    .move_in_relative(displacement.v().scale(distance));
            }

            // -mouse_delta[0] is because I the positive direction on the mouse
            // is right
            self.camera
                .pose
                .look_left(-mouse_delta[0] * self.sensitivity);

            // -mouse_delta[1] is because I have inverted Y
            self.camera
                .pose
                .look_up(-mouse_delta[1] * self.sensitivity, None);
        }
    }
}
