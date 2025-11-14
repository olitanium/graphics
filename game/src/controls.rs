use std::collections::HashSet;

use engine::linear_algebra::Vector;
use engine::modelling::TestLineVertex;
use engine::{ApiError, ColourRGB, EngResult, Key};

use crate::state::{State, StateError};

impl State {
    pub fn controls(
        &mut self,
        mouse_delta: (f32, f32),
        keyboard: HashSet<Key>,
        string: &mut String,
        frame_time: f32,
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
                    self.line.as_mut().push(TestLineVertex {
                        position: Vector::new([self.angle.cos(), 0.0, self.angle.sin()]),
                        colour: ColourRGB::new([self.angle.cos(), 0.0, self.angle.sin()]),
                    });
                    self.angle += 0.1;
                }
                'b' | 'B' => self.show_bbs = !self.show_bbs,
                'm' | 'M' => self.draw_models = !self.draw_models,
                'o' => {
                    self.bounding_box_level = self.bounding_box_level.wrapping_add(1);
                    // if let Some(bbox) = self.imported.bounding_box() {
                    // self.imported_bounding_box_model = bbox
                    // .make_model(self.bounding_box_level,
                    // Rc::new(Material::white())) .build();
                    // }
                }
                'i' => {
                    self.bounding_box_level = self.bounding_box_level.wrapping_sub(1);
                    // if let Some(bbox) = self.imported.bounding_box() {
                    // self.imported_bounding_box_model = bbox
                    // .make_model(self.bounding_box_level,
                    // Rc::new(Material::white())) .build();
                    // }
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
