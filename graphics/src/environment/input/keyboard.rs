use alloc::rc::Rc;
use core::cell::{RefCell, RefMut};
use std::collections::HashSet;

pub use glfw::{Action, Key};

#[derive(Debug, Default)]
struct Inner {
    keys: HashSet<Key>,
    buffer: String,
}

#[derive(Debug)]
pub struct Keyboard(Rc<RefCell<Inner>>);

impl Keyboard {
    pub(crate) fn new(window: &mut glfw::Window) -> Self {
        let out = Self(Rc::default());

        let key_ptr = Rc::clone(&out.0);
        let backspace_ptr = Rc::clone(&out.0);
        let buffer_ptr = Rc::clone(&out.0);

        window.set_key_callback(move |_, key, _, action, _| {
            match action {
                Action::Press => {
                    key_ptr.borrow_mut().keys.insert(key);
                }
                Action::Release => {
                    key_ptr.borrow_mut().keys.remove(&key);
                }
                Action::Repeat => {}
            }

            match (key, action) {
                // Perfom on Press or Repeat
                (Key::Backspace, Action::Press | Action::Repeat) => {
                    backspace_ptr.borrow_mut().buffer.pop();
                }
                (Key::Enter, Action::Press | Action::Repeat) => {
                    backspace_ptr.borrow_mut().buffer.push('\n');
                }
                // Do not act on other keys
                _ => {}
            }
        });

        window.set_char_callback(move |_, ch| {
            buffer_ptr.borrow_mut().buffer.push(ch);
        });

        out
    }

    pub(crate) fn get_buffer(&self) -> RefMut<'_, String> {
        RefMut::map(self.0.borrow_mut(), |inner| &mut inner.buffer)
    }

    pub fn get_depressed_keys(&self) -> HashSet<Key> {
        self.0.borrow().keys.clone()
    }
}
