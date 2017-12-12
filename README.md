# What is This?
[![Build](https://travis-ci.org/shawnscode/crayon.svg?branch=master)](https://travis-ci.org/shawnscode/crayon)
[![Documentation](https://docs.rs/crayon/badge.svg)](https://docs.rs/crayon)
[![Crate](https://img.shields.io/crates/v/crayon.svg)](https://crates.io/crates/crayon)
[![Downloads](https://img.shields.io/crates/d/crayon.svg)](https://crates.io/crates/crayon)
[![License](https://img.shields.io/crates/l/crayon.svg)](https://github.com/shawnscode/crayon/blob/master/LICENSE-APACHE)

Crayon is an experimental purpose game engine, written with a minimalistic modular design philosophy. Its built from the ground up to focus on cache friendly data layouts in multicore environments with entity-component based architecture.

It is loosely inspired by some amazing blogs on [bitsquid](https://bitsquid.blogspot.de) and [molecular](https://blog.molecular-matters.com). Some goals include:

- Stateless, layered, multithread render system with OpenGL(ES) 2.0+ backends.
- Entity component system with a data-driven designs.
- Flexible workflow with default supports for some common resources.
- Ease scripts integration with Lua.
- etc.

*Warning*: This project is a work in progress and far from a stable version right now.

### Quick Example
For the sake of brevity, you can also run a simple and quick example with commands:

``` sh
git clone git@github.com:shawnscode/crayon.git && cd crayon/crayon-examples
cargo run imgui
```

### Examples and Screenshots

Check out [here](./crayon-examples) for details.

![ImGui](./crayon-examples/screenshots/imgui.png)