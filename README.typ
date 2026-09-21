#title[A 3D graphics rending library]

This project aims to teach me all about 3D graphics, linear algebra, quaternions and other concepts with the aim of making a simple-to-use 3D rending suite. I have tried to implement everything myself, using external crates only where it would be infeasible to make them myself *yet* such as `glfw` and `gl`.

The primary design motivation amongst this project is to ensure that a program with draw errors *would not compile*. The design of the graphics engine is based on that of OpenGL, but with wiggle room to extend to Vulkan once I have gotten my head around it. I do not intend to support any other APIs.

The project is split into a large number of crates, which are successively re-exported up to the `graphics` crate which is the logical first crate to look at. The crate defines a lot of useful logical drawable units such as models, quads and skyboxes. Some crates are not actually yet included in the tree, such as `local_rc` which is untested.

= Documentation

The documentation in this project is poor and is a current priority fix - as well as ensuring that documentation (and doc-tests) are written in parallel with code. Documentaion efforts will start with the user-facing components and will expand to private functions afterwards.

= Clippy

The very strict Clippy in the root `Cargo.toml` does not pass, this is also a current priority fix.

= How to start using?

There is a `template_app` which can be used to begin the development of a 3D render, this template shows a dummy implementation of the `ApiState` trait, reads events and submits a `DrawNothing` to the graphics environment.