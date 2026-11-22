#![allow(clippy::unwrap_used, reason = "demo")]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]
#![feature(const_closures)]
#![feature(const_trait_impl)]

extern crate alloc;
use engine::{Environment, EnvironmentConfiguration};

mod camera;
mod controls;
mod new;
mod physics;
mod player;
mod prep_draw;
mod state;
mod sun;

fn main() {
    Environment::<state::State>::new(&EnvironmentConfiguration {
        gl_version: (3, 3),
        screen_dims: (2000, 1000),
        full_screen: false,
        title: "Window",
        mouse_fix_to_centre: true,
    })
    .unwrap()
    .run()
    .unwrap();
}
