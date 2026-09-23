extern crate alloc;
mod state;
use state::State;

#[expect(
    clippy::expect_used,
    reason = "fails when cannot create window or state"
)]
fn main() {
    let env = engine::Environment::<State>::new((3, 3), (500, 500), "triangle_test", false)
        .expect("should not fail to create window or `State`");
    env.run().expect("Should not fail");
}
