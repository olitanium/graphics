use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use glfw::PWindow;
pub use glfw::{CursorMode, MouseButton as Button};

use super::Action;

// First mouse means no movement this check
// Upon change, set the change flag to true
// Upon read, if change flag is true, proceed, else skip
//     revert change flag,
//     read mouse data,
//     calculate difference,
//     set old position to current,
//     return difference

#[derive(Debug, Clone, Copy, Default)]
pub enum MouseState {
    #[default]
    FirstMouse,
    Moved {
        current_location: (f64, f64),
        delta: (f64, f64),
    },
    Stationary {
        current_location: (f64, f64),
    },
}

#[derive(Default, Debug)]
struct Inner {
    state: Cell<MouseState>,
    buttons: RefCell<HashSet<Button>>,
}

#[derive(Debug)]
pub struct Mouse(Rc<Inner>);

impl Mouse {
    pub(crate) fn new(window: &mut PWindow, fix_to_centre: bool) -> Self {
        let out = Mouse(Rc::default());
        let state_ptr = out.0.clone();

        window.set_cursor_pos_callback(move |_, x, y| {
            let y = -y;

            let new = match state_ptr.state.get() {
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
                // TODO: MOVED => STATIONARY
            };

            state_ptr.state.set(new);
        });

        let buttons_ptr = out.0.clone();

        window.set_mouse_button_callback(move |_, mouse_button, action, _| match action {
            Action::Press => {
                buttons_ptr.buttons.borrow_mut().insert(mouse_button);
            }
            Action::Release => {
                buttons_ptr.buttons.borrow_mut().remove(&mouse_button);
            }
            Action::Repeat => {}
        });

        if fix_to_centre {
            window.set_cursor_mode(CursorMode::Disabled);
        }

        out
    }

    pub(crate) fn get_delta(&self) -> (f64, f64) {
        let out = match self.0.state.get() {
            MouseState::Moved { delta, .. } => delta,
            _ => (0.0, 0.0),
        };

        if let MouseState::Moved {
            current_location, ..
        } = self.0.state.get()
        {
            self.0
                .state
                .set(MouseState::Stationary { current_location });
        }

        out
    }

    pub(crate) fn get_position(&self) -> (f64, f64) {
        match self.0.state.get() {
            MouseState::FirstMouse => (0.0, 0.0),
            MouseState::Stationary { current_location } => current_location,
            MouseState::Moved {
                current_location, ..
            } => current_location,
        }
    }

    pub(crate) fn get_buttons_depressed(&self) -> HashSet<Button> {
        self.0.buttons.borrow().clone()
    }
}
