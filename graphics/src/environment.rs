use std::collections::HashSet;

mod draw;
mod global_state;
mod input;
mod window;

pub use draw::Draw;
use glfw::fail_on_errors;
pub use global_state::ApiState;
pub use input::keyboard::Key;
pub use input::mouse::Button;
use utils::error_boilerplate;
pub use window::Window;

use crate::error::ApiResult;
use crate::framebuffer::{DefaultFramebuffer, FramebufferContext};
use crate::shader_program::ShaderProgramContext;
use crate::types::TexDim;
use crate::ApiError;

#[derive(Debug, Clone)]
pub enum Error {
    GlfwInit { glfw_error: glfw::InitError },
    GlfwWindow,
}

error_boilerplate!(Error);

impl<E> From<Error> for crate::error::ApiError<E> {
    fn from(value: Error) -> Self {
        Self::Window(value)
    }
}

use std::ffi::{CStr, c_void};
#[expect(unused_variables)]
extern "system" fn debug_callback(
    source: u32,
    gltype: u32,
    id: u32,
    severity: u32,
    length: i32,
    message: *const i8,
    user_param: *mut c_void,
) {
    let message = unsafe { CStr::from_ptr(message) };
    let source = match source {
        gl::DEBUG_SOURCE_API => "DEBUG_SOURCE_API",
        gl::DEBUG_SOURCE_WINDOW_SYSTEM => "DEBUG_SOURCE_WINDOW_SYSTEM",
        gl::DEBUG_SOURCE_SHADER_COMPILER => "DEBUG_SOURCE_SHADER_COMPILER",
        gl::DEBUG_SOURCE_THIRD_PARTY => "DEBUG_SOURCE_THIRD_PARTY",
        gl::DEBUG_SOURCE_APPLICATION => "DEBUG_SOURCE_APPLICATION",
        gl::DEBUG_SOURCE_OTHER => "DEBUG_SOURCE_OTHER",
        _ => "ERROR_SOURCE_NOT_FOUND",
    };
    let gltype = match gltype {
        gl::DEBUG_TYPE_ERROR => "DEBUG_TYPE_ERROR",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "DEBUG_TYPE_DEPRECATED_BEHAVIOR",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "DEBUG_TYPE_UNDEFINED_BEHAVIOR",
        gl::DEBUG_TYPE_PORTABILITY => "DEBUG_TYPE_PORTABILITY",
        gl::DEBUG_TYPE_PERFORMANCE => "DEBUG_TYPE_PERFORMANCE",
        gl::DEBUG_TYPE_MARKER => "DEBUG_TYPE_MARKER",
        gl::DEBUG_TYPE_PUSH_GROUP => "DEBUG_TYPE_PUSH_GROUP",
        gl::DEBUG_TYPE_POP_GROUP => "DEBUG_TYPE_POP_GROUP",
        gl::DEBUG_TYPE_OTHER => "DEBUG_TYPE_OTHER",
        _ => "ERROR TYPE NOT FOUND",
    };
    let severity = match severity {
        gl::DEBUG_SEVERITY_HIGH => "DEBUG_SEVERITY_HIGH",
        gl::DEBUG_SEVERITY_MEDIUM => "DEBUG_SEVERITY_MEDIUM",
        gl::DEBUG_SEVERITY_LOW => "DEBUG_SEVERITY_LOW",
        gl::DEBUG_SEVERITY_NOTIFICATION => return, //"DEBUG_SEVERITY_NOTIFICATION",
        _ => "ERROR SEVERITY NOT FOUND",
    };

    println!(
        "GL Callback:\n\tsource =  \t{source}\n\ttype =    \t{gltype}\n\tseverity \
         =\t{severity}\n\tmessage = \t{message:#?}",
    );
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Event<'a> {
    WindowBuffer(&'a DefaultFramebuffer),
    FrameTime(f64),
    ActualTime(f64),
    WindowResize((TexDim, TexDim)),
    TextBuffer(&'a mut String),
    Keyboard(HashSet<Key>),
    Mouse {
        buttons: HashSet<Button>,
        position: (f64, f64),
        delta: (f64, f64),
    },
}

#[derive(Debug)]
pub struct Environment<G: ApiState> {
    global_state: G,
    window: Window,
    old_frame: f64,
    glfw: glfw::Glfw,
}

impl<G: ApiState> Environment<G> {
    pub fn new(
        gl_version: (u32, u32),
        screen_dims: (u32, u32),
        title: &str,
        mouse_fix_to_centre: bool,
        // initial_state: G,
    ) -> ApiResult<Self, G::Error> {
        let mut glfw =
            glfw::init(fail_on_errors!()).map_err(|glfw_error| Error::GlfwInit { glfw_error })?;

        let (gl_major, gl_minor) = gl_version;
        glfw.window_hint(glfw::WindowHint::ContextVersionMajor(gl_major));
        glfw.window_hint(glfw::WindowHint::ContextVersionMinor(gl_minor));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));

        let mut window = Window::builder()
            .suppress_config(true)
            .dims(screen_dims)
            .mouse_fix_to_centre(mouse_fix_to_centre)
            .title(title)
            .raw_motion(true)
            .glfw(&mut glfw)
            .build().map_err(ApiError::temp_convert_from_never)?;


        glfw.set_swap_interval(glfw::SwapInterval::None);

        // we need a GL context before we can load OpenGL functions
        gl::load_with(|s| glfw.get_proc_address_raw(s));

        window.configure();

        // let window2 = Window::builder()
        // .dims(screen_dims)
        // .share_with(&window)
        // .mouse_fix_to_centre(mouse_fix_to_centre)
        // .title(title)
        // .raw_motion(true)
        // .build()?;

        let global_state = G::new(window.get_framebuffer_size())?;

        Ok(Self {
            glfw,
            window,
            // window2,
            global_state,

            old_frame: 0.0,
        })
    }

    pub fn run(&mut self) -> ApiResult<(), G::Error> {
        // Substitute for `for to_draw in env.iter() {`
        // self.window.make_current();

        let mut shader_program_marker =
            ShaderProgramContext::new().expect("First invocation means only one marker");
        let mut framebuffer_register =
            FramebufferContext::new().expect("First invocation means only one register");

        let mut frame_iter = self.iter();
        while let Some(result) = frame_iter.next() {
            match result {
                Ok(to_draw) => {
                    framebuffer_register.clear();
                    // Begin rendering code
                    for draw in to_draw {
                        draw.draw(&mut framebuffer_register, &mut shader_program_marker);
                    }
                }
                Err(error) => return Err(error),
            }
            //}
        }

        Ok(())
    }

    fn poll(&mut self) -> ApiResult<Vec<Box<dyn Draw + '_>>, G::Error> {
        let curr_time = self.glfw.get_time();
        let frametime = curr_time - self.old_frame;
        self.old_frame = curr_time;

        let window = &mut self.window;
        let mut events = window.poll_events();

        events.push(Event::ActualTime(curr_time));
        events.push(Event::FrameTime(frametime));

        self.global_state.poll(events)
    }

    #[expect(clippy::iter_not_returning_iterator)]
    fn iter(&mut self) -> FrameIter<'_, G> {
        FrameIter::new(self)
    }
}

struct FrameIter<'a, G: ApiState> {
    env: &'a mut Environment<G>,
}

impl<'env, G: ApiState> FrameIter<'env, G> {
    fn new(env: &'env mut Environment<G>) -> Self {
        Self { env }
    }
}

impl<G: ApiState> FrameIter<'_, G> {
    pub fn next(&mut self) -> Option<ApiResult<Vec<Box<dyn Draw + '_>>, G::Error>> {
        if !self.env.window.should_close() {
            // Poll for and process events
            self.env.glfw.poll_events();
            // Swap front and back buffers
            self.env.window.swap_buffers();

            match self.env.poll() {
                Err(crate::error::ApiError::Close) => None,
                x => Some(x),
            }
        } else {
            None
        }
    }
}
