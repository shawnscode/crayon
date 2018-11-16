# What is This?
[![Build status](https://travis-ci.org/shawnscode/crayon.svg?branch=master)](https://travis-ci.org/shawnscode/crayon)
[![Codecov](https://codecov.io/gh/shawnscode/crayon/branch/master/graph/badge.svg)](https://codecov.io/gh/shawnscode/crayon)
[![Documentation](https://docs.rs/crayon/badge.svg)](https://docs.rs/crayon)
[![Crate](https://img.shields.io/crates/v/crayon.svg)](https://crates.io/crates/crayon)
[![License](https://img.shields.io/crates/l/crayon.svg)](https://github.com/shawnscode/crayon/blob/master/LICENSE-APACHE)

Crayon is a small, portable and extensible game framework, which loosely inspired by some amazing blogs on [bitsquid](https://bitsquid.blogspot.de), [molecular](https://blog.molecular-matters.com) and [floooh](http://floooh.github.io/).

Some goals include:

- Intuitive c style (lifetime free) interfaces and extensible through external code modules;
- Run on PCs, Mobiles and Web browsers from the same source;
- Stateless, layered, multithread render system with OpenGL(ES) 3.0 or WebGL 2.0 backend;
- Simplified assets workflow and asynchronous data loading from various filesystem;
- Unified interfaces for handling input devices across platforms;
- Built from the ground up to focus on multi-thread friendly with a work-stealing job scheduler;
- etc.

This project adheres to [Semantic Versioning](http://semver.org/), all notable changes will be documented in this [file](./CHANGELOG.md).

### Assets Workflow

The asset workflow comes with the version 0.5.0. During the development, the assets could be stored in formats which could producing and editing by authoring tools directly, and it will be compiled into some kind of effecient format for runtime (which is dependent on platform and hardware devices usually).

The assets manipulation codes are placed under [crayon-tools](https://github.com/shawnscode/crayon-tools), checks out the repository for further details.

### Quick Example

For the sake of brevity, you can als run a simple and quick example with commands:

``` sh
git clone git@github.com:shawnscode/crayon.git && cd crayon/examples
cargo run --bin render_texture
```

### Platform-Specific

