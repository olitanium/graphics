use core::cell::RefMut;
use core::error::Error;
use core::fmt::{self, Debug};
use core::time::Duration;
use std::collections::HashSet;
use std::time::Instant;

mod draw;
mod global_state;
mod input;
mod window;

pub use draw::{Draw, DrawNothing, DrawOut};
use glfw::fail_on_errors;
pub use global_state::ApiState;
pub use input::keyboard::Key;
pub use input::mouse::Button;
use utils::error_boilerplate;
pub use window::Window;

pub use crate::environment::draw::PollResult;
use crate::environment::window::GlfwError;
use crate::framebuffer::{DefaultFramebuffer, FramebufferContext};
use crate::gl_call;
use crate::linear_algebra::Vector;
use crate::shader_program::ShaderProgramContext;
use crate::types::{Query, TexDim};

#[non_exhaustive]
pub enum NewError<G: ApiState> {
    GlfwInit(glfw::InitError),
    GlfwWindow(GlfwError),
    StateInit(G::Error),
}
impl<G: ApiState> Debug for NewError<G>
where
    G::Error: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GlfwInit(x) => f.debug_tuple("NewError::GlfwInit").field(x).finish(),
            Self::GlfwWindow(x) => f.debug_tuple("NewError::GlfwWindow").field(x).finish(),
            Self::StateInit(x) => f.debug_tuple("NewError::StateInit").field(x).finish(),
        }
    }
}
error_boilerplate!({G: ApiState} NewError<G>);

use core::ffi::{CStr, c_void};
#[expect(
    unused_variables,
    reason = "naming is important, this is an extern function"
)]
extern "system" fn debug_callback(
    source: u32,
    gltype: u32,
    id: u32,
    severity: u32,
    length: i32,
    message: *const i8,
    user_param: *mut c_void,
) {
    // SAFETY:
    // This pointer came from the OpenGL API, so must be assumed well formed.
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

    eprintln!(
        "GL Callback:\n\tsource =  \t{source}\n\ttype =    \t{gltype}\n\tseverity \
         =\t{severity}\n\tmessage = \t{message:#?}",
    );
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Event<'a> {
    WindowBuffer(&'a DefaultFramebuffer),
    FrameTime(Duration),
    ElapsedTime(Duration),
    WindowResize((TexDim, TexDim)),
    Keyboard {
        keys: HashSet<Key>,
        buffer: RefMut<'a, String>,
    },
    Mouse {
        buttons: HashSet<Button>,
        position: Vector<2>,
        delta: Vector<2>,
    },
    Query(Vec<(usize, Query)>),
}

#[derive(Debug)]
pub struct Environment<G: ApiState> {
    global_state: G,
    window: Window,
    old_frame: Option<Instant>,
    creation_time: Instant,
    glfw: glfw::Glfw,
}

impl<G: ApiState> Environment<G> {
    pub fn new(
        gl_version: (u32, u32),
        screen_dims: (u32, u32),
        title: &str,
        mouse_fix_to_centre: bool,
    ) -> Result<Self, NewError<G>> {
        let mut glfw =
            glfw::init(fail_on_errors!()).map_err(|glfw_error| NewError::GlfwInit(glfw_error))?;

        let (gl_major, gl_minor) = gl_version;
        glfw.window_hint(glfw::WindowHint::ContextVersionMajor(gl_major));
        glfw.window_hint(glfw::WindowHint::ContextVersionMinor(gl_minor));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

        let mut window = Window::builder()
            .suppress_config(true)
            .dims(screen_dims)
            .mouse_fix_to_centre(mouse_fix_to_centre)
            .title(title.to_owned())
            .raw_motion(true)
            .glfw(&mut glfw)
            .build()
            .map_err(|err| NewError::GlfwWindow(err))?;

        glfw.set_swap_interval(glfw::SwapInterval::None);

        // we need a GL context, created above before we can load OpenGL
        // functions
        gl::load_with(|s| glfw.get_proc_address_raw(s));

        let version_ptr = gl_call! { gl::GetString(gl::VERSION) };
        // SAFETY:
        // The pointer returned is guaranteed to be from the GPU and have a Nul
        // terminator.
        let version_cstring = unsafe { CStr::from_ptr(version_ptr.cast()) };

        eprintln!("OpenGL Version: {version_cstring:?}");

        window.configure();

        let global_state = G::new(window.size()).map_err(|err| NewError::StateInit(err))?;

        let creation_time = Instant::now();

        Ok(Self {
            glfw,
            window,
            global_state,
            creation_time,
            old_frame: None,
        })
    }
}

pub enum RunError<G: ApiState> {
    State(G::Error),
    DrawError(Box<dyn Error>),
    ContextError,
}
error_boilerplate!({G: ApiState} RunError<G>);

impl<G: ApiState> fmt::Debug for RunError<G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContextError => f.debug_struct("RunError::ContextError").finish(),
            Self::State(g) => f.debug_struct(&format!("RunError::State({g:?})")).finish(),
            Self::DrawError(e) => f
                .debug_struct(&format!("RunError::DrawError({e:?})"))
                .finish(),
        }
    }
}

impl<G: ApiState> Environment<G> {
    pub fn run(mut self) -> Result<(), RunError<G>> {
        let mut shader_program_marker =
            ShaderProgramContext::new().ok_or(RunError::ContextError)?;
        let mut framebuffer_context = FramebufferContext::new().ok_or(RunError::ContextError)?;

        // Substitute for `for to_draw in env.iter() {`
        loop {
            let mut frame_iter = self.iter();

            let mut query_outputs = None;

            loop {
                let result = frame_iter.next(query_outputs.take());
                match result {
                    PollResult::Close => return Ok(()),
                    PollResult::Error(err) => return Err(RunError::State(err)),
                    PollResult::Draw(to_draw) => {
                        framebuffer_context.clear();
                        query_outputs = to_draw
                            .draw_sized(&mut framebuffer_context, &mut shader_program_marker)
                            .map_err(RunError::DrawError)?
                            .get_query();
                    }
                    PollResult::Restart => break,
                }
            }

            let framebuffer_size = self.window.size();
            self.global_state = G::new(framebuffer_size).map_err(RunError::State)?;
        }
    }

    fn poll(
        &mut self,
        queries: Option<Vec<(usize, Query)>>,
    ) -> PollResult<G::DrawOutput<'_>, G::Error> {
        let curr_time = Instant::now();
        let frametime = curr_time - *self.old_frame.get_or_insert(curr_time);
        self.old_frame = Some(curr_time);

        let window = &mut self.window;
        let mut events = window.poll_events();

        events.push(Event::ElapsedTime(curr_time - self.creation_time));
        events.push(Event::FrameTime(frametime));
        if let Some(queries) = queries {
            events.push(Event::Query(queries));
        }

        self.global_state.poll(events)
    }

    #[expect(
        clippy::iter_not_returning_iterator,
        reason = "this is a `BorrowingIterator`"
    )]
    const fn iter(&mut self) -> FrameIter<'_, G> {
        FrameIter::new(self)
    }
}

struct FrameIter<'a, G: ApiState> {
    env: &'a mut Environment<G>,
}

impl<'env, G: ApiState> FrameIter<'env, G> {
    const fn new(env: &'env mut Environment<G>) -> Self {
        Self { env }
    }
}

impl<G: ApiState> FrameIter<'_, G> {
    pub fn next(
        &mut self,
        queries: Option<Vec<(usize, Query)>>,
    ) -> PollResult<G::DrawOutput<'_>, G::Error> {
        if self.env.window.should_close() {
            PollResult::Close
        } else {
            // Poll for and process events
            self.env.glfw.poll_events();
            // Swap front and back buffers
            self.env.window.swap_buffers();

            self.env.poll(queries)
        }
    }
}
