use std::collections::HashSet;

use glfw::PWindow;
pub use glfw::{CursorMode, MouseButton as Button};

use crate::input::Action;

// First mouse means no movement this check
// Upon change, set the change flag to true
// Upon read, if change flag is true, proceed, else skip
//     revert change flag,
//     read mouse data,
//     calculate difference,
//     set old position to current,
//     return difference

#[derive(Debug, Clone, Copy)]
pub enum MouseState {
    FirstMouse,
    Moved {
        current_location: (f64, f64),
        delta: (f64, f64),
    },
    Stationary {
        current_location: (f64, f64),
    },
}

#[derive(Debug)]
struct Inner {
    state: MouseState,
    buttons: HashSet<Button>,
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            state: MouseState::FirstMouse,
            buttons: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct Mouse(Box<Inner>);

impl Mouse {
    #[inline]
    pub(crate) fn new(window: &mut PWindow, fix_to_centre: bool) -> Self {
        let mut out = Mouse(Box::default());

        let state_ptr = &raw mut out.0.state;

        window.set_cursor_pos_callback(move |_, x, y| {
            let y = -y;
            // SAFETY: unsure
            unsafe { state_ptr.as_mut() }.map(|data| {
                *data = match *data {
                    MouseState::FirstMouse => MouseState::Stationary {
                        current_location: (x, y),
                    },
                    MouseState::Stationary { current_location } => MouseState::Moved {
                        current_location: (x, y),
                        delta: (x - current_location.0, y - current_location.1),
                    },
                    MouseState::Moved {
                        current_location,
                        delta,
                    } => MouseState::Moved {
                        current_location: (x, y),
                        delta: (
                            x - current_location.0 + delta.0,
                            y - current_location.1 + delta.1,
                        ),
                    },
                }
            });
        });

        let buttons_ptr = &raw mut out.0.buttons;

        window.set_mouse_button_callback(move |_, mouse_button, action, _| match action {
            Action::Press => {
                unsafe { buttons_ptr.as_mut() }.map(|data| data.insert(mouse_button));
            }
            Action::Release => {
                unsafe { buttons_ptr.as_mut() }.map(|data| data.remove(&mouse_button));
            }
            Action::Repeat => {}
        });

        if fix_to_centre {
            window.set_cursor_mode(CursorMode::Disabled);
        }

        out
    }

    pub(crate) fn get_delta(&mut self) -> (f64, f64) {
        let out = match self.0.state {
            MouseState::Moved { delta, .. } => delta,
            _ => (0.0, 0.0),
        };

        self.0.state = match self.0.state {
            MouseState::Moved {
                current_location, ..
            } => MouseState::Stationary { current_location },
            other => other,
        };

        out
    }

    pub(crate) fn get_position(&self) -> (f64, f64) {
        match self.0.state {
            MouseState::FirstMouse => (0.0, 0.0),
            MouseState::Stationary { current_location } => current_location,
            MouseState::Moved {
                current_location, ..
            } => current_location,
        }
    }

    pub(crate) fn get_buttons_depressed(&self) -> HashSet<Button> {
        self.0.buttons.clone()
    }
}
