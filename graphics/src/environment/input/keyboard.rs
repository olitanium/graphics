use std::cell::RefCell;
use std::collections::HashSet;

use glfw::PWindow;
pub use glfw::{Action, Key};

#[derive(Debug, Default)]
struct Inner {
    keys: HashSet<Key>,
    buffer: RefCell<String>,
}

#[derive(Debug)]
pub struct Keyboard(Box<Inner>);

impl Keyboard {
    /// UNSURE OF SAFETY
    pub(crate) fn new(window: &mut PWindow) -> Self {
        let mut out = Self(Box::default());

        let key_ptr = &raw mut out.0.keys;
        let backspace_ptr = &raw const out.0.buffer;
        let buffer_ptr = &raw const out.0.buffer;

        window.set_key_callback(move |_, key, _, action, _| {
            match action {
                Action::Press => unsafe { key_ptr.as_mut() }.map(|x| x.insert(key)),
                Action::Release => unsafe { key_ptr.as_mut() }.map(|x| x.remove(&key)),
                _ => None,
            };

            match (key, action) {
                // Intenionally ignore ALL releases
                (_, Action::Release) => {}
                // Perfom on Press or Repeat
                (Key::Backspace, _) => {
                    unsafe { backspace_ptr.as_ref() }.map(|x| x.borrow_mut().pop());
                }
                (Key::Enter, _) => {
                    unsafe { backspace_ptr.as_ref() }.map(|x| x.borrow_mut().push('\n'));
                }
                // Do not act on other keys
                _ => {}
            }
        });

        window.set_char_callback(move |_, char| {
            unsafe { buffer_ptr.as_ref() }.map(|x| x.borrow_mut().push(char));
            unsafe { buffer_ptr.as_ref() }.map(|x| eprintln!("{}", x.borrow_mut()));
        });

        out
    }

    pub(crate) fn get_reset_buffer(&mut self) -> String {
        self.0.buffer.take()
    }

    pub fn get_depressed_keys(&self) -> HashSet<Key> {
        self.0.keys.clone()
    }
}
