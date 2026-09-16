use std::collections::HashSet;
use std::ops::{FromResidual, Residual, Try};
use std::time::Duration;

use engine::linear_algebra::Vector;
use engine::{ApiState, Key, PollResult};

use crate::state::{State, StateError};

pub enum ControlOutput<T> {
    Data(T),
    Error(StateError),
    Close,
    Restart,
}

impl<T> Residual<T> for ControlOutput<!> {
    type TryType = ControlOutput<T>;
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

impl<T, E: Into<StateError>> FromResidual<Result<!, E>> for ControlOutput<T> {
    fn from_residual(residual: Result<!, E>) -> Self {
        let Err(x) = residual;
        ControlOutput::Error(x.into())
    }
}

impl FromResidual<ControlOutput<!>>
    for PollResult<<State as ApiState>::DrawOutput<'_>, StateError>
{
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
        frame_time: Duration,
        _time: Duration,
    ) -> ControlOutput<()> {
        let frame_time = f32::min(frame_time.as_secs_f32(), 1.0 / 60.0);

        if keyboard.contains(&Key::LeftControl) {
            return ControlOutput::Restart;
        }
        if keyboard.contains(&Key::Escape) {
            return ControlOutput::Close;
        }
        if keyboard.contains(&Key::R) {
            self.exposure -= 1.2 * frame_time;
        }
        if keyboard.contains(&Key::E) {
            self.exposure += 1.2 * frame_time;
        }

        if let Some(ch) = string.pop() {
            match ch {
                'Q' => string.clear(),
                'b' | 'B' => self.toggle_b = !self.toggle_b,
                'm' | 'M' => self.toggle_m = !self.toggle_m,
                'g' | 'G' => self.toggle_g = !self.toggle_g,
                'h' | 'H' => self.toggle_h = !self.toggle_h,
                ch => string.push(ch),
            }
        }

        if string.ends_with('q') | string.ends_with('Q') {
            _ = string.pop();
            self.game_camera.move_lock = !self.game_camera.move_lock;
        }
        self.game_camera
            .controls(&keyboard, string, mouse_delta, frame_time);

        let small_cube_anim = self.small_cube.debug_get_pair_model_bind_pose().1;
        let origin_cube_anim = self.origin_cube.debug_get_pair_model_bind_pose().1;

        if !self.game_camera.move_lock {
            self.player.controls(
                &self.player.get_pair_model().1,
                &self.game_camera,
                &keyboard,
                frame_time,
                &mut self.small_cube,
                &small_cube_anim,
                &mut self.origin_cube,
                &origin_cube_anim,
            );
        }

        ControlOutput::Data(())
    }
}
