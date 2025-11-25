#![feature(array_windows)]
#![feature(type_changing_struct_update)]
#![expect(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(array_try_map)]
#![feature(try_blocks)]
#![feature(never_type)]
#![feature(option_array_transpose)]
#![feature(associated_const_equality)]
#![feature(result_option_map_or_default)]
#![feature(iter_map_windows)]
#![feature(associated_type_defaults)]
#![deny(unused_must_use)]
#![deny(clippy::missing_const_for_fn)]

pub mod eng_state;
pub mod entity;
#[expect(hidden_glob_reexports)]
mod error;
pub mod modelling;
pub mod opengl_shaders;
pub use array_vec;
pub use entity::Entity;
pub use error::{EngError, EngResult};
pub use graphics::colour::{ColourRGB, ColourRGBA};
pub use graphics::framebuffer::Builder as FrameBufferBuilder;
pub use graphics::{types, *};
pub use russimp::scene::PostProcess;
mod transform;
pub use transform::TransformBy;

pub type VecDraw<'a, E> = Vec<Box<dyn Draw<EngError<E>> + 'a>>;
