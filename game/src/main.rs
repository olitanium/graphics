#![expect(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use engine::Environment;

mod camera;
mod controls;
mod morgan;
mod new;
mod physics;
mod player;
mod prep_draw;
mod state;

fn main() {
    Environment::<state::State>::new((4, 5), (2000, 1000), "Window", true)
        .unwrap()
        .run()
        .unwrap();
}
