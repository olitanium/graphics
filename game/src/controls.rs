use std::collections::HashSet;
use std::convert::Infallible;
use std::mem;
use std::ops::{FromResidual, Try};
use std::time::Duration;

use engine::entity::SkeletonEntity;
use engine::linear_algebra::Vector;
use engine::modelling::TestLineVertex;
use engine::modelling::cubic::geometry::Pose;
use engine::{ColourRGB, Key, PollResult};

use crate::morgan::Sun;
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
        frame_time: Duration,
        _time: Duration,
    ) -> ControlOutput<()> {
        if keyboard.contains(&Key::LeftControl) && keyboard.contains(&Key::Num0) {
            return ControlOutput::Restart;
        }
        if keyboard.contains(&Key::Escape) {
            return ControlOutput::Close;
        }
        if keyboard.contains(&Key::R) {
            self.exposure -= 1.2 * frame_time.as_secs_f32();
        }
        if keyboard.contains(&Key::E) {
            self.exposure += 1.2 * frame_time.as_secs_f32();
        }

        if let Some(ch) = string.pop() {
            match ch {
                'Q' => string.clear(),
                'b' | 'B' => self.toggle_b = !self.toggle_b,
                'm' | 'M' => {
                    self.toggle_m = !self.toggle_m;
                }
                'g' | 'G' => self.toggle_g = !self.toggle_g,
                'h' | 'H' => self.toggle_h = !self.toggle_h,
                other => string.push(other),
            }
        }

        if string.ends_with('q') | string.ends_with('Q') {
            _ = string.pop();
            self.game_camera.move_lock = !self.game_camera.move_lock;
        }
        self.game_camera
            .controls(keyboard, string, mouse_delta, frame_time);

        ControlOutput::Data(())
    }
}
