# A 3D graphics rending library (Rust + OpenGL)

This project aims to teach me all about 3D graphics, linear algebra, quaternions and other concepts with the aim of making a simple-to-use 3D rending suite. I have tried to implement everything myself, using external crates only where it would be infeasible to make them myself *yet* such as `glfw` and `gl`.

The primary design motivation for this project is to ensure that a program with draw errors *would not compile*. The design of the graphics engine is based on the workflow of OpenGL, but with wiggle room to extend to Vulkan once I have gotten my head around it. I do not currently intend to support any other APIs.

The project is split into a large number of crates, which are successively re-exported up to the `graphics` crate which would be the logical first crate to read. The crate defines a lot of useful logical drawable units such as models, quads and skyboxes. Some crates are not actually yet included in the tree, such as `local_rc` which is untested.

## Why I built this

I built this as a way to learn both low level Rust and low level graphics rendering. I wanted to best leverage the Rust type system to make it impossible for a library user to draw something that would fail slently or at runtime. This goal is still in development, as there are many places in the graphicas pipeline where there can be issues.

## Getting Started

There is a `template_app` which can be used to begin the development of a 3D render, this template shows a dummy implementation of the `ApiState` trait, reads events and submits a `DrawNothing` to the graphics environment.

## Examples

* `triangle_test` - Minimal triangle rendered with transformations and collision detection
* `grappling_hook` - Interactive scene with collision and constraints
* `box_with_sun` - Lit scene with multiple light sources and skybox.

run with

```bash
cargo run --release --bin <OPTION>
```

### With Nix (recommended)

`nix develop` to enter the flake environment with `cargo`.

### On Windows

Untested, requires `glfw3`, `assimp` and `OpenGL` libraries.

## Known issues and WIP

* Documentation: sparse; high priority for expansion
* Clippy: stict lints such as `must_use_candidate` and `private_doc` not yet fulfilled
* Windows Support: Not tested

## Roadmap

* Clippy compliance
* Expanded Documentation
