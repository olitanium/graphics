#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

extern crate alloc;

use engine::{Environment, EnvironmentConfiguration};

mod controls;
mod new;
mod physics;
mod prep_draw;
mod state;

fn main() {
    let config = EnvironmentConfiguration {
        gl_version: (4, 5),
        screen_dims: (2000, 1000),
        ..Default::default()
    };
    Environment::<state::State>::new(&config)
        .unwrap()
        .run()
        .unwrap();
}
