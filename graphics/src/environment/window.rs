use std::cell::Cell;
use std::ptr;
use std::rc::Rc;

use glfw::{Context, Glfw, GlfwReceiver, PWindow, WindowEvent};
use utils::{builder, error_boilerplate, new};

use super::input::keyboard::Keyboard;
use super::input::mouse::{CursorMode, Mouse};
use crate::framebuffer::DefaultFramebuffer;
use crate::types::TexDim;
use crate::{Event, gl_call};

#[expect(dead_code)]
#[derive(Debug)]
pub struct Window {
    glfw_window: PWindow, // DefaultFramebuffer,
    default_framebuffer: DefaultFramebuffer,
    events: GlfwReceiver<(f64, WindowEvent)>,
    pub(crate) window_resized: Rc<Cell<bool>>,
    pub(crate) keyboard: Keyboard,
    pub(crate) mouse: Mouse,
}

impl Window {
    pub fn builder() -> Builder<MissingGlfw> {
        Builder::new()
    }

    pub(crate) fn get_framebuffer_size(&self) -> (TexDim, TexDim) {
        let (width, height) = self.glfw_window.get_framebuffer_size();
        (TexDim::new(width), TexDim::new(height))
    }

    pub(crate) fn window_resized(&self) -> bool {
        self.window_resized.get()
    }

    pub(crate) fn should_close(&self) -> bool {
        self.glfw_window.should_close()
    }

    pub(crate) fn swap_buffers(&mut self) {
        self.glfw_window.swap_buffers();
    }

    pub(crate) fn poll_events(&mut self) -> Vec<Event<'_>> {
        let mut event_buffer = Vec::new();

        let mouse = Event::Mouse {
            buttons: self.mouse.get_buttons_depressed(),
            position: self.mouse.get_position(),
            delta: self.mouse.get_delta(),
        };
        event_buffer.push(mouse);

        if self.window_resized() {
            event_buffer.push(Event::WindowResize(self.get_framebuffer_size()));
        }

        let keyboard = Event::Keyboard {
            keys: self.keyboard.get_depressed_keys(),
            buffer: self.keyboard.get_buffer(),
        };
        event_buffer.push(keyboard);

        event_buffer.push(Event::WindowBuffer(&self.default_framebuffer));

        event_buffer
    }

    pub fn set_cursor_mode(&mut self, mode: CursorMode) {
        self.glfw_window.set_cursor_mode(mode);
    }

    pub fn configure(&mut self) {
        gl_call! {
            gl::Enable(gl::BLEND);
        };
        gl_call! {
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        };

        gl_call! {
            gl::Enable(gl::DEBUG_OUTPUT);
        };
        gl_call! {
            gl::DebugMessageCallback(Some(crate::environment::debug_callback), ptr::null());
        };

        gl_call! {
            gl::ClearColor(1.0, 0.0, 0.1, 1.0);
        };

        gl_call! {
            gl::Enable(gl::LINE_SMOOTH);
        };
    }
}

#[derive(Debug, Default)]
pub struct MissingGlfw;
#[derive(Debug)]
pub struct HasGlfw<'a>(&'a mut Glfw);

#[derive(Debug)]
pub struct Builder<G> {
    maker: G,
    suppress_config: bool,
    dims: (u32, u32),
    mouse_fix_to_centre: bool,
    title: String,
    raw_motion: bool,
    windowed: bool,
}

impl Default for Builder<MissingGlfw> {
    fn default() -> Self {
        Self {
            maker: MissingGlfw,
            suppress_config: false,
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

    pub fn glfw(self, glfw: &mut Glfw) -> Builder<HasGlfw<'_>> {
        Builder {
            maker: HasGlfw(glfw),
            ..self
        }
    }
}

impl<G> Builder<G> {
    builder!(mouse_fix_to_centre: bool);

    builder!(suppress_config: bool);

    builder!(title: String);

    builder!(dims: (u32, u32));

    builder!(raw_motion: bool);

    builder!(windowed: bool);
}

#[derive(Debug)]
pub struct GlfwError;
error_boilerplate!(GlfwError);

impl Builder<HasGlfw<'_>> {
    pub(crate) fn build(self) -> Result<Window, GlfwError> {
        // Create a windowed mode window and its OpenGL context
        let (glfw_window, events) = self
            .maker
            .0
            .create_window(
                self.dims.0,
                self.dims.1,
                &self.title,
                if self.windowed {
                    glfw::WindowMode::Windowed
                } else {
                    // TODO: Fullscreen
                    todo!("Full Screen not yet supported")
                },
            )
            .ok_or(GlfwError)?;
        Ok(make_window(self, glfw_window, events))
    }
}

fn make_window<T>(
    builder: Builder<T>,
    mut glfw_window: PWindow,
    events: GlfwReceiver<(f64, WindowEvent)>,
) -> Window {
    glfw_window.make_current();
    glfw_window.set_key_polling(true);

    let window_resized = Rc::new(Cell::new(false));

    let resized_ptr = window_resized.clone();
    glfw_window
        .set_framebuffer_size_callback(move |_window, _width, _height| resized_ptr.set(true));

    if glfw_window.glfw.supports_raw_motion() && builder.raw_motion {
        glfw_window.set_raw_mouse_motion(true);
    }

    let keyboard = Keyboard::new(&mut glfw_window);
    let mouse = Mouse::new(&mut glfw_window, builder.mouse_fix_to_centre);

    let size = glfw_window.get_framebuffer_size();
    let size = (TexDim::new(size.0), TexDim::new(size.1));
    let default_framebuffer = DefaultFramebuffer::new(size);

    let mut window = Window {
        window_resized,
        default_framebuffer,
        events,
        glfw_window,
        keyboard,
        mouse,
    };

    if !builder.suppress_config {
        window.configure();
    }

    window
}
