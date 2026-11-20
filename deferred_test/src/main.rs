#![allow(clippy::unwrap_used, reason = "demo")]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![feature(const_closures)]
#![feature(const_trait_impl)]

extern crate alloc;
use engine::Environment;

mod camera;
mod controls;
mod new;
mod physics;
mod player;
mod prep_draw;
mod state;
mod sun;

fn main() {
    Environment::<state::State>::new((3, 3), (2000, 1000), "Window", true)
        .unwrap()
        .run()
        .unwrap();
}
