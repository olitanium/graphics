mod state;
use state::State;

fn main() {
    let env =
        engine::Environment::<State>::new((3, 3), (500, 500), "triangle_test", false).unwrap();
    _ = env.run();
}
