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
#![feature(generic_const_items)]
#![feature(bool_to_result)]
#![deny(unused_must_use)]
#![deny(clippy::missing_const_for_fn)]
#![deny(missing_debug_implementations)]

pub mod eng_state;
pub mod entity;
pub mod modelling;
pub mod opengl_shaders;
pub use array_vec;
pub use entity::SimpleEntity;
pub use graphics::colour::{ColourRGB, ColourRGBA};
pub use graphics::framebuffer::Builder as FrameBufferBuilder;
pub use graphics::{types, *};
pub use russimp::scene::PostProcess;
mod transform;
pub use transform::TransformBy;
pub use transform::Transform;

pub type VecDraw<'a> = Vec<Box<dyn Draw + 'a>>;
