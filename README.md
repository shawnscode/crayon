### Build Status
[![Build Status](https://travis-ci.org/kaisc/crayon.svg?branch=master)](https://travis-ci.org/kaisc/crayon)
[![Crate Status](http://meritbadge.herokuapp.com/crayon)](https://crates.io/crates/crayon)

### Introduction
Crayon is an experimental purpose project, written with a minimalistic modular design philosophy. Its built from the ground up to focus on cache friendly data layouts in multicore environments with entity-component based architecture.

It is loosely inspired by some amazing blogs on [bitsquid](https://bitsquid.blogspot.de) and [molecular](https://blog.molecular-matters.com). Some goals include:

- Stateless, layered, multithread render system with OpenGL(ES) 2.0+ backends.
- Entity component system with a data-driven designs.
- Flexible workflow with default supports for some common resources.
- Ease scripts integration with Lua.
- etc.

*Warning*: This project is a work in progress and far from a stable version right now.

### Usage (Workflow)
The most recommanded way to work with `crayon` is following commands below:

``` sh
git clone git@github.com:kaisc/crayon.git
cd crayon
cargo install --path crayon-cli --force
```

If everything goes well, you should get a CLI tool named `crayon-cli`. Its barely useable right now, and only supports basic project creation and resource manipulations. In spite of the unstable status of this project, feel free to checkout and build to follow progress recently.

### Quick Example
For the sake of brevity, you can also run a simple and quick example with commands:

``` sh
git clone git@github.com:kaisc/crayon.git
cd crayon
cargo run --manifest-path crayon-runtime/Cargo.toml --example sprite
```

<p align="center">
  <img src="screenshots/sprite-particles.gif">
</p>

### FAQ

#### Why Rust ?

First of all, this is a part-time toy project of myself,  so i don't really care if we ever have a game engine written in Rust.

And the most importantly, it makes sense for me to take the numerous advantages of modern programming language, instead of using languages like C/C++ which have many historical burden. In fact, this project has beed development with C++ for about four months, which produce a basic multi-thread, and much more pains.

#### Why should we have a module named `crayon-workflow`?

Builds a game project is not only about compile the source files into binary, it should take care of enormous trivial tasks. Its common to handle the pre-processing of resources and archiving with some kind of shell scripts, but its always tedious and error-prone. Its vital to address a flexible and robust mechanism to handle them for real-world game engine.

As a result of that, we provide a basic workflow framework `crayon-workflow`, and a simple command line interface (CLI) `crayon-cli`. It should be easy to make your own workflow with it.