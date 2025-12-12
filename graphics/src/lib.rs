#![expect(incomplete_features)]
#![feature(debug_closure_helpers)]
#![feature(string_from_utf8_lossy_owned)]
#![feature(generic_const_exprs)]
#![feature(stmt_expr_attributes)]
#![feature(type_changing_struct_update)]
#![feature(box_as_ptr)]
#![feature(lazy_type_alias)]
#![feature(inherent_associated_types)]
#![feature(never_type)]
#![feature(option_array_transpose)]
#![feature(generic_const_items)]
#![feature(try_trait_v2)]
#![feature(unwrap_infallible)]

mod environment;
pub mod shader_program;
pub mod texture;
pub mod types;

pub mod element_array_buffer;
pub mod framebuffer;
pub mod model;
pub mod vertex;
pub mod vertex_array;
pub mod vertex_buffer;

pub(crate) use environment::PrivateToken;
pub use environment::{ApiState, Draw, DrawNothing, DrawOut, Environment, Event, Key, PollResult};
pub use model::Model;
pub use shader_program::{ActiveShaderProgram, ShaderProgram, ShaderProgramContext};
pub use {colour, linear_algebra};

pub mod query;

#[macro_export]
macro_rules! gl_call_inner {
    ($input:stmt) => {{
        // Skip all previous errors which have been ignored
        while unsafe { gl::GetError() } != gl::NO_ERROR {}
        // perform the expression
        #[allow(clippy::macro_metavars_in_unsafe)]
        let output = unsafe {
            //#[expect(clippy::macro_metavars_in_unsafe)]
            $input
        };
        // read through errors, returning Err if there are many.
        let errors: Vec<$crate::types::GLError> =
            std::iter::repeat_with(|| $crate::types::GLError(unsafe { gl::GetError() }))
                .take_while(|error| error.0 != gl::NO_ERROR)
                .collect();
        if errors.is_empty() {
            // Ok(output)
        } else {
            // Err(
            panic!("{:?}", errors)
            //)
        } //.unwrap()
        output
    }};
}

#[macro_export]
macro_rules! gl_call {
    ($input:stmt) => {{ $crate::gl_call_inner!($input) }};

    ($input:stmt;) => {{
        $crate::gl_call_inner!($input);
    }};
}
