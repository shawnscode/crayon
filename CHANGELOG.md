# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog][kc], and this project adheres to
[Semantic Versioning][sv].

[kc]: http://keepachangelog.com/
[sv]: http://semver.org/

## [Unreleased]

* Decouple `glutin` from the core window system.
* Add `VariantStr/VariantVec` for small heap object optimization.
* Address internal texture format based on OpenGL version and profile.
* Replace `HashMap/HashSet` with `FastHashMap/FastHashSet` type which based on `FxHasher` currently.
* Add profile scripts.

## [0.5.1] - 2018-09-01
* Re-export macros from cgmath.
* Add optional attribute field in shader.
* Fix un-expected side-effects of GLVisitor::bind_texture.

## 0.5.0 - 2018-08-13
* Rebase the initial release from v0.5.0.

[Unreleased]: https://github.com/shawnscode/crayon/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/shawnscode/crayon/compare/v0.5.0...v0.5.1