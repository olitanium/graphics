#![feature(iter_chain)]
#![feature(array_windows)]
#![feature(type_changing_struct_update)]
#![feature(generic_const_exprs)]
#![feature(array_chunks)]
#![feature(array_try_map)]

pub mod modelling;
pub mod opengl_shaders;
mod error;

pub use graphics::colour::{ ColourRGB, ColourRGBA };
pub use array_vec;
pub use graphics::buffers::Builder as FrameBufferBuilder;
pub use graphics::types;
pub use graphics::*;
pub use russimp::scene::PostProcess;

