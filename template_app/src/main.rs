mod state;
use engine::EnvironmentConfiguration;
use state::State;

#[expect(
    clippy::expect_used,
    reason = "fails when cannot create window or state"
)]
fn main() {
    let env = engine::Environment::<State>::new(&EnvironmentConfiguration {
        gl_version: (3, 3),
        screen_dims: (2000, 1000),
        full_screen: false,
        title: "Window",
        mouse_fix_to_centre: true,
    })
    .expect("should not fail to create window or `State`");
    env.run().expect("Should not fail");
}
