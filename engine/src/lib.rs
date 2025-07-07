#![feature(iter_chain)]
#![feature(array_windows)]
#![feature(type_changing_struct_update)]
#![expect(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(array_chunks)]
#![feature(array_try_map)]

#[expect(hidden_glob_reexports)]
mod error;
pub mod modelling;
pub mod opengl_shaders;

pub use array_vec;
pub use graphics::colour::{ColourRGB, ColourRGBA};
pub use graphics::framebuffer::Builder as FrameBufferBuilder;
pub use graphics::{types, *};
pub use russimp::scene::PostProcess;
