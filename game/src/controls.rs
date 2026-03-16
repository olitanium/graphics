use std::collections::HashSet;
use std::convert::Infallible;
use std::mem;
use std::ops::{FromResidual, Try};

use engine::entity::SkeletonEntity;
use engine::linear_algebra::Vector;
use engine::modelling::TestLineVertex;
use engine::modelling::cubic::geometry::Pose;
use engine::{ColourRGB, Key, PollResult};

use crate::morgan::Morgan;
use crate::state::{State, StateError};

pub enum ControlOutput<T> {
    Data(T),
    Error(StateError),
    Close,
    Restart,
}

impl<T> Try for ControlOutput<T> {
    type Output = T;
    type Residual = ControlOutput<!>;

    fn branch(self) -> std::ops::ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::Data(x) => std::ops::ControlFlow::Continue(x),
            Self::Close => std::ops::ControlFlow::Break(ControlOutput::Close),
            Self::Error(e) => std::ops::ControlFlow::Break(ControlOutput::Error(e)),
            Self::Restart => std::ops::ControlFlow::Break(ControlOutput::Restart),
        }
    }

    fn from_output(output: Self::Output) -> Self {
        Self::Data(output)
    }
}

impl<T> FromResidual for ControlOutput<T> {
    fn from_residual(residual: <Self as Try>::Residual) -> Self {
        match residual {
            ControlOutput::Close => Self::Close,
            ControlOutput::Error(e) => Self::Error(e),
            ControlOutput::Restart => Self::Restart,
        }
    }
}

impl<T, E: Into<StateError>> FromResidual<Result<Infallible, E>> for ControlOutput<T> {
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        match residual {
            Err(x) => ControlOutput::Error(x.into()),
        }
    }
}

impl FromResidual<ControlOutput<!>> for PollResult<'_, StateError> {
    fn from_residual(residual: ControlOutput<!>) -> Self {
        match residual {
            ControlOutput::Close => PollResult::Close,
            ControlOutput::Error(err) => PollResult::Error(err),
            ControlOutput::Restart => PollResult::Restart,
        }
    }
}

impl State {
    pub fn controls(
        &mut self,
        mouse_delta: Vector<2>,
        keyboard: HashSet<Key>,
        string: &mut String,
        frame_time: f32,
        _time: f32,
    ) -> ControlOutput<()> {
        if keyboard.contains(&Key::LeftControl) && keyboard.contains(&Key::Num0) {
            return ControlOutput::Restart;
        }
        if keyboard.contains(&Key::Escape) {
            return ControlOutput::Close;
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
                    self.grappling_hook.start();
                    // self.reset_time = time;
                }
                'g' | 'G' => self.toggle_g = !self.toggle_g,
                'o' => {
                    self.bounding_box_level = self.bounding_box_level.wrapping_add(1);
                }
                'i' => {
                    self.bounding_box_level = self.bounding_box_level.wrapping_sub(1);
                }
                't' => {
                    if let Some(Morgan {
                        inner: morgan_model,
                    }) = mem::take(&mut self.morgan)
                    {
                        self.grappling_hook.entity.remove(
                            self.grappling_hook
                                .hook
                                .take()
                                .expect("will be replaced later"),
                        );
                        let new_pos_index = self
                            .grappling_hook
                            .entity
                            .push(
                                SkeletonEntity::from(morgan_model),
                                Pose::new_from_translation(Vector::new([0.0, 0.0, 0.5])),
                                &self.grappling_hook.chain,
                                &self.grappling_hook.chain_end_bone,
                            )
                            .unwrap();

                        eprintln!(
                            "old: {:?}, new: {:?}",
                            self.grappling_hook.hook, new_pos_index
                        );

                        self.grappling_hook.hook = Some(new_pos_index);
                    }
                }

                other => string.push(other),
            }
        }

        self.camera
            .pose
            .look_left(-mouse_delta[0] * self.sensitivity);
        self.camera.pose.look_up(mouse_delta[1] * self.sensitivity);

        ControlOutput::Data(())
    }
}
