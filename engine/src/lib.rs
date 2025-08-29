#![feature(iter_chain)]
#![feature(array_windows)]
#![feature(type_changing_struct_update)]
#![expect(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(array_try_map)]
#![feature(try_blocks)]
#![feature(never_type)]
#![feature(option_array_transpose)]
#![feature(associated_const_equality)]

pub mod eng_state;
#[expect(hidden_glob_reexports)]
mod error;
pub mod modelling;
pub mod opengl_shaders;

pub use array_vec;
pub use error::{EngApiResult, EngError, EngResult};
pub use graphics::colour::{ColourRGB, ColourRGBA};
pub use graphics::framebuffer::Builder as FrameBufferBuilder;
pub use graphics::{types, *};
pub use russimp::scene::PostProcess;

pub type VecDraw<'a> = Vec<Box<dyn Draw + 'a>>;
