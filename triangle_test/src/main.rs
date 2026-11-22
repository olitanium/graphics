#![expect(clippy::unwrap_used, reason = "demo")]
#![feature(const_ops)]
#![feature(const_trait_impl)]

extern crate alloc;
mod state;
use engine::EnvironmentConfiguration;
use state::State;

fn main() {
    engine::Environment::<State>::new(&EnvironmentConfiguration {
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
