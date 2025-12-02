#![feature(never_type)]
#![feature(result_option_map_or_default)]
#![expect(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(array_windows)]

use engine::eng_state::EngState;
use engine::{EngResult, Environment};

mod controls;
mod new;
mod physics;
mod prep_draw;
mod state;

fn main() -> EngResult<(), <state::State as EngState>::Error> {
    Environment::<state::State>::new((4, 5), (2000, 1000), "Window", true)?.run()?;

    Ok(())
}
