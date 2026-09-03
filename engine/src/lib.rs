#![expect(incomplete_features, reason = "Nightly used")]
#![feature(type_changing_struct_update)]
//#![feature(generic_const_exprs)]
#![feature(array_try_map)]
#![feature(never_type)]
#![feature(option_array_transpose)]
#![feature(min_generic_const_args)]
#![feature(macroless_generic_const_args)]
#![feature(iter_map_windows)]
#![feature(inherent_associated_types)]
#![feature(const_trait_impl)]
#![feature(iter_collect_into)]
#![feature(const_destruct)]
#![feature(slice_partition_dedup)]

pub mod eng_state;
pub mod entity;
pub mod modelling;
pub mod opengl_shaders;
pub use array_vec;
pub use entity::SimpleEntity;
pub use geometry;
pub use graphics::colour::{ColourRGB, ColourRGBA};
pub use graphics::framebuffer::Builder as FrameBufferBuilder;
pub use graphics::{types, *};
pub use russimp::scene::PostProcess;

pub type VecDraw = Vec<Box<dyn Draw>>;
