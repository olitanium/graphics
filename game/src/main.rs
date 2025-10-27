#![feature(never_type)]
#![feature(result_option_map_or_default)]
#![expect(incomplete_features)]
#![feature(generic_const_exprs)]

use engine::Environment;

mod controls;
mod new;
mod physics;
mod prep_draw;
mod state;

fn main() {
    Environment::<state::State>::new((4, 5), (1920, 1080), "Window", true)
        .unwrap()
        .run()
        .unwrap()
}
