#![expect(incomplete_features, reason = "Nightly used")]
#![feature(debug_closure_helpers)]
#![feature(type_changing_struct_update)]
#![feature(checked_type_aliases)]
#![feature(inherent_associated_types)]
#![feature(option_array_transpose)]
#![feature(try_trait_v2_residual)]
#![feature(try_trait_v2)]
#![feature(unwrap_infallible)]
#![feature(macroless_generic_const_args)]
#![feature(min_generic_const_args)]
#![feature(array_try_map)]
#![feature(const_trait_impl)]

extern crate alloc;

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

pub use colour;
pub use environment::{ApiState, Draw, DrawNothing, DrawOut, Environment, Event, Key, PollResult};
pub use linear_algebra;
pub use model::Model;
pub use shader_program::{ActiveShaderProgram, ShaderProgram, ShaderProgramContext};

pub mod query;

#[macro_export]
macro_rules! gl_call_inner {
    ($input:stmt) => {{
        // Skip all previous errors which have been ignored
        // SAFETY: gl::GetError() does not do anything unsafe.
        while unsafe { gl::GetError() } != gl::NO_ERROR {}
        // perform the expression
        #[expect(
            clippy::allow_attributes,
            reason = "Some calls will not require unsafety"
        )]
        #[allow(
            clippy::macro_metavars_in_unsafe,
            reason = "gl_call is itself an unsafety marker"
        )]
        #[expect(clippy::undocumented_unsafe_blocks, reason = "FFI")]
        let output = unsafe { $input };
        // read through errors, returning Err if there are many.
        let errors: Vec<$crate::types::GLError> = core::iter::repeat_with(|| {
            $crate::types::GLError(
                // SAFETY: gl::GetError() does not do anything unsafe.
                unsafe { gl::GetError() },
            )
        })
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
