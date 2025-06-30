use opengl::environment::Environment;

mod controls;
mod new;
mod state;

fn main() {
    Environment::<state::State>::new((3, 3), (1920, 1080), "Window", true)
        .unwrap()
        .run()
        .unwrap()
}
