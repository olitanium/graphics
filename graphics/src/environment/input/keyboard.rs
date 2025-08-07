use std::cell::RefCell;
use std::collections::HashSet;
use std::marker::PhantomPinned;
use std::pin::Pin;

use glfw::PWindow;
pub use glfw::{Action, Key};

#[derive(Debug, Default)]
struct Inner {
    keys: RefCell<HashSet<Key>>,
    buffer: RefCell<String>,
    _phantom_pinned: PhantomPinned,
}

#[derive(Debug)]
pub struct Keyboard(Pin<Box<Inner>>);

impl Keyboard {
    pub(crate) fn new(window: &mut PWindow) -> Self {
        let out = Self(Box::pin(Default::default()));

        let key_ptr = &raw const out.0.keys;
        let backspace_ptr = &raw const out.0.buffer;
        let buffer_ptr = &raw const out.0.buffer;

        const ERROR_MESSAGE: &str =
            "Pointer was created based on box, and so is valid for reads and writes";
        window.set_key_callback(move |_, key, _, action, _| {
            match action {
                Action::Press => {
                    unsafe { key_ptr.as_ref() }
                        .expect(ERROR_MESSAGE)
                        .borrow_mut()
                        .insert(key);
                }
                Action::Release => {
                    unsafe { key_ptr.as_ref() }
                        .expect(ERROR_MESSAGE)
                        .borrow_mut()
                        .remove(&key);
                }
                _ => {}
            };

            match (key, action) {
                // Perfom on Press or Repeat
                (Key::Backspace, Action::Press | Action::Repeat) => {
                    unsafe { backspace_ptr.as_ref() }
                        .expect(ERROR_MESSAGE)
                        .borrow_mut()
                        .pop();
                }
                (Key::Enter, Action::Press | Action::Repeat) => {
                    unsafe { backspace_ptr.as_ref() }
                        .expect(ERROR_MESSAGE)
                        .borrow_mut()
                        .push('\n');
                }
                // Do not act on other keys
                _ => {}
            }
        });

        window.set_char_callback(move |_, char| {
            unsafe { buffer_ptr.as_ref() }
                .expect(ERROR_MESSAGE)
                .borrow_mut()
                .push(char);
        });

        out
    }

    pub(crate) fn get_buffer(&mut self) -> &mut String {
        // SAFETY: the inner_ref never moves the RefCell. A mut ref to the buffer
        // is released, but it's movement is not possible
        let inner_ref = unsafe { self.0.as_mut().get_unchecked_mut() };

        inner_ref.buffer.get_mut()
    }

    pub fn get_depressed_keys(&self) -> HashSet<Key> {
        self.0.keys.borrow().clone()
    }
}
