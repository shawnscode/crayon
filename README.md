# What is This?
[![Build](https://travis-ci.org/shawnscode/crayon.svg?branch=master)](https://travis-ci.org/shawnscode/crayon)
[![Build status](https://ci.appveyor.com/api/projects/status/ced1ds3ud2h8u4ut?svg=true)](https://ci.appveyor.com/project/shawnscode/crayon)
[![Documentation](https://docs.rs/crayon/badge.svg)](https://docs.rs/crayon)
[![Crate](https://img.shields.io/crates/v/crayon.svg)](https://crates.io/crates/crayon)
[![Downloads](https://img.shields.io/crates/d/crayon.svg)](https://crates.io/crates/crayon)
[![License](https://img.shields.io/crates/l/crayon.svg)](https://github.com/shawnscode/crayon/blob/master/LICENSE-APACHE)

Crayon is an experimental purpose game engine, written with a minimalistic modular design philosophy. Its built from the ground up to focus on cache friendly data layouts in multicore environments with entity-component based architecture.

It is loosely inspired by some amazing blogs on [bitsquid](https://bitsquid.blogspot.de) and [molecular](https://blog.molecular-matters.com). Some goals include:

- Extensible through external code modules;
- Run on macOS, Linux, Windows, iOS, Android, WebAssembly from the same source;
- Stateless, layered, multithread render system with OpenGL(ES) 2.0+ backends;
- Entity component system with a data-driven designs;
- Unified access to input devices across platforms;
- Asynchronous data loading from various filesystem;
- etc.

This project adheres to [Semantic Versioning](http://semver.org/), all notable changes will be documented in this [file](./CHANGELOG.md).

### Quick Example
For the sake of brevity, you can also run a simple and quick example with commands:

``` sh
git clone git@github.com:shawnscode/crayon.git && cd crayon/crayon-examples
cargo run imgui
```

### Examples and Screenshots

Check out [here](./crayon-examples) for details.

![ImGui](./crayon-examples/screenshots/imgui.png)