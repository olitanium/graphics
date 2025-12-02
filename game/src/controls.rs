use std::collections::HashSet;
use std::mem;

use engine::linear_algebra::Vector;
use engine::modelling::TestLineVertex;
use engine::modelling::cubic::geometry::Pose;
use engine::{ApiError, ColourRGB, EngResult, Key, TransformBy};

use crate::state::{State, StateError};

impl State {
    pub fn controls(
        &mut self,
        mouse_delta: (f32, f32),
        keyboard: HashSet<Key>,
        string: &mut String,
        frame_time: f32,
        time: f32,
    ) -> EngResult<(), StateError> {
        if keyboard.contains(&Key::Escape) {
            return Err(ApiError::Close.into());
        }
        if keyboard.contains(&Key::W) {
            self.camera.pose.move_forward(self.speed[2] * frame_time);
        }
        if keyboard.contains(&Key::A) {
            self.camera.pose.move_left(self.speed[0] * frame_time);
        }
        if keyboard.contains(&Key::D) {
            self.camera.pose.move_left(-self.speed[0] * frame_time)
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
        // if keyboard.contains(&Key::Right) {
        // let diff = Pose::new_from_translation(Vector::new([0.0, 0.0, frame_time *
        // 0.5])); self.morgan.pose = self.morgan.pose.transform_by(&diff);
        // }
        // if keyboard.contains(&Key::Left) {
        // let diff = Pose::new_from_translation(Vector::new([0.0, 0.0, -frame_time *
        // 0.5])); self.morgan.pose = self.morgan.pose.transform_by(&diff);
        // }
        // if keyboard.contains(&Key::Up) {
        // let diff = Pose::new_from_translation(Vector::new([0.0, frame_time * 0.5,
        // 0.0])); self.morgan.pose = self.morgan.pose.transform_by(&diff);
        // }
        // if keyboard.contains(&Key::Down) {
        // let diff = Pose::new_from_translation(Vector::new([0.0, -frame_time * 0.5,
        // 0.0])); self.morgan.pose = self.morgan.pose.transform_by(&diff);
        // }
        if keyboard.contains(&Key::U) {
            self.camera.radius_out(-0.5 * frame_time);
        }
        if keyboard.contains(&Key::J) {
            self.camera.radius_out(0.5 * frame_time);
        }
        // if string.to_lowercase().contains('y') {
        // self.which_animation = (self.which_animation + 1) % 2
        // }
        if keyboard.contains(&Key::Comma) {
            self.camera.pose.roll_ccw(frame_time);
        }
        if keyboard.contains(&Key::Period) {
            self.camera.pose.roll_ccw(-frame_time);
        }

        if let Some(ch) = string.pop() {
            match ch {
                'Q' => string.clear(),
                'l' => {
                    self.line.get_mut().push(TestLineVertex {
                        position: Vector::new([self.angle.cos(), 0.0, self.angle.sin()]),
                        colour: ColourRGB::new([self.angle.cos(), 0.0, self.angle.sin()]),
                    });
                    self.angle += 0.1;
                }
                'b' | 'B' => self.toggle_b = !self.toggle_b,
                'm' | 'M' => {
                    self.toggle_m = !self.toggle_m;
                    self.reset_time = time;
                }
                'g' | 'G' => self.toggle_g = !self.toggle_g,
                'o' => {
                    self.bounding_box_level = self.bounding_box_level.wrapping_add(1);
                }
                'i' => {
                    self.bounding_box_level = self.bounding_box_level.wrapping_sub(1);
                }
                't' => {
                    self.grappling_hook.pop(self.hook_pos);
                    if let Some(morgan_model) = mem::take(&mut self.morgan) {
                    let (attachment_point, bone_index) = self.join_pos;
                    self.hook_pos = self
                        .grappling_hook
                        .push_simple_entity(
                            morgan_model,
                            Pose::new_from_translation(Vector::new([0.0, 0.0, 0.5])),
                            attachment_point,
                            bone_index,
                        )
                        .unwrap()
                    }
                }

                other => string.push(other),
            }
        }

        self.camera
            .pose
            .look_left(-mouse_delta.0 * self.sensitivity);
        self.camera.pose.look_up(mouse_delta.1 * self.sensitivity);

        Ok(())
    }
}
