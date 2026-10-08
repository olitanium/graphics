use alloc::rc::Rc;
use core::cell::{Cell, RefCell};
use std::collections::HashSet;

use glfw::PWindow;
pub use glfw::{CursorMode, MouseButton as Button};
use linear_algebra::{Vector, vector};

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
        current_location: Vector<2>,
        delta: Vector<2>,
    },
    Stationary {
        current_location: Vector<2>,
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
        let out = Self(Rc::default());
        let state_ptr = Rc::clone(&out.0);

        window.set_cursor_pos_callback(move |_, x, y| {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "no need for such precision"
            )]
            let y = -y as f32;
            #[expect(
                clippy::cast_possible_truncation,
                reason = "no need for such precision"
            )]
            let x = x as f32;

            let new = match state_ptr.state.get() {
                MouseState::FirstMouse => MouseState::Stationary {
                    current_location: vector![x, y],
                },
                MouseState::Stationary { current_location } => {
                    let [curr0, curr1] = current_location.into_inner();
                    MouseState::Moved {
                        current_location: vector![x, y],
                        delta: vector![x - curr0, y - curr1],
                    }
                }
                MouseState::Moved {
                    current_location,
                    delta,
                } => {
                    let [delta_x, delta_y] = delta.into_inner();
                    let [curr_x, curr_y] = current_location.into_inner();
                    MouseState::Moved {
                        current_location: vector![x, y],
                        delta: vector![x - curr_x + delta_x, y - curr_y + delta_y],
                    }
                }
            };

            state_ptr.state.set(new);
        });

        let buttons_ptr = Rc::clone(&out.0);

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

    pub(crate) fn get_delta(&self) -> Vector<2> {
        let out = match self.0.state.get() {
            MouseState::Moved { delta, .. } => delta,
            MouseState::FirstMouse | MouseState::Stationary { .. } => vector![0.0, 0.0],
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

    pub(crate) fn get_position(&self) -> Vector<2> {
        match self.0.state.get() {
            MouseState::FirstMouse => vector![0.0, 0.0],
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
