#![feature(never_type)]
#![feature(result_option_map_or_default)]
#![expect(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(array_windows)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_residual)]

use engine::Environment;

mod controls;
mod grappling_hook;
mod morgan;
mod new;
mod physics;
mod prep_draw;
mod properties;
mod state;

fn main() {
    Environment::<state::State>::new((4, 5), (2000, 1000), "Window", true)
        .unwrap()
        .run()
        .unwrap();
}
