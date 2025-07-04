use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use utils::{builder, new};

use super::input::keyboard::Keyboard;
use super::input::mouse::{CursorMode, Mouse};
use crate::environment::Error;
use crate::error::Result;
use crate::framebuffer::DefaultFramebuffer;
use crate::types::TexDim;

#[expect(dead_code)]
#[derive(Debug)]
pub struct Window {
    glfw_window: PWindow,
    pub(crate) default_framebuffer: DefaultFramebuffer,
    events: GlfwReceiver<(f64, WindowEvent)>,
    window_resized: Box<bool>,
    keyboard: Keyboard,
    mouse: Mouse,
}

impl Window {
    #[must_use]
    #[inline]
    pub fn builder() -> Builder<MissingGlfw> {
        Builder::new()
    }

    #[must_use]
    #[inline]
    pub(crate) fn get_framebuffer_size(&self) -> (TexDim, TexDim) {
        let (width, height) = self.glfw_window.get_framebuffer_size();
        (TexDim::new(width), TexDim::new(height))
    }

    pub(crate) fn mouse(&self) -> &Mouse {
        &self.mouse
    }

    pub(crate) fn mouse_mut(&mut self) -> &mut Mouse {
        &mut self.mouse
    }

    pub(crate) fn keyboard(&self) -> &Keyboard {
        &self.keyboard
    }

    pub(crate) fn keyboard_mut(&mut self) -> &mut Keyboard {
        &mut self.keyboard
    }

    pub(crate) fn window_resized(&self) -> bool {
        *self.window_resized
    }

    #[must_use]
    #[inline]
    pub(crate) fn should_close(&self) -> bool {
        self.glfw_window.should_close()
    }

    #[inline]
    pub(crate) fn swap_buffers(&mut self) {
        self.glfw_window.swap_buffers();
    }

    #[inline]
    pub fn set_cursor_mode(&mut self, mode: CursorMode) {
        self.glfw_window.set_cursor_mode(mode);
    }
}

#[derive(Debug, Default)]
pub struct MissingGlfw;
#[derive(Debug)]
pub struct HasGlfw<'a>(&'a mut Glfw);

#[derive(Debug)]
pub struct Builder<G> {
    glfw: G,
    dims: (u32, u32),
    mouse_fix_to_centre: bool,
    title: String,
    raw_motion: bool,
    windowed: bool,
}

impl Default for Builder<MissingGlfw> {
    fn default() -> Self {
        Self {
            glfw: MissingGlfw,
            dims: (480, 360),
            mouse_fix_to_centre: false,
            title: String::new(),
            raw_motion: false,
            windowed: true,
        }
    }
}

impl Builder<MissingGlfw> {
    new!();

    pub fn glfw(self, glfw: &mut Glfw) -> Builder<HasGlfw> {
        Builder {
            glfw: HasGlfw(glfw),
            ..self
        }
    }
}

impl<G> Builder<G> {
    builder!(mouse_fix_to_centre: bool);

    builder!(title: String);

    builder!(dims: (u32, u32));

    builder!(raw_motion: bool);

    builder!(windowed: bool);
}

impl Builder<HasGlfw<'_>> {
    pub(crate) fn build(self) -> Result<Window> {
        // Create a windowed mode window and its OpenGL context
        let (mut glfw_window, events) = self
            .glfw
            .0
            .create_window(
                self.dims.0,
                self.dims.1,
                &self.title,
                if self.windowed {
                    glfw::WindowMode::Windowed
                } else {
                    todo!("Full Screen not yet supported")
                },
            )
            .ok_or_else(|| Error::GlfwWindowError)?;

        glfw_window.make_current();
        glfw_window.set_key_polling(true);

        let mut window_resized = Box::new(false);

        let resized_ptr = Box::as_mut_ptr(&mut window_resized);
        glfw_window.set_framebuffer_size_callback(move |_window, _width, _height| unsafe {
            *resized_ptr = true
        });

        if self.glfw.0.supports_raw_motion() && self.raw_motion {
            glfw_window.set_raw_mouse_motion(true);
        }

        let keyboard = Keyboard::new(&mut glfw_window);
        let mouse = Mouse::new(&mut glfw_window, self.mouse_fix_to_centre);

        let size = glfw_window.get_framebuffer_size();
        let size = (TexDim::new(size.0), TexDim::new(size.1));
        let default_framebuffer = DefaultFramebuffer::new(size);

        Ok(Window {
            window_resized,
            default_framebuffer,
            events,
            glfw_window,
            keyboard,
            mouse,
        })
    }
}
