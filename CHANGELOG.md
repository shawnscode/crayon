# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog][kc], and this project adheres to
[Semantic Versioning][sv].

[kc]: http://keepachangelog.com/
[sv]: http://semver.org/

## [Unreleased]

### Fixed
* Fixed unexcepted panics when closing window. ([#40])

### Changed
* Introduced `Mesh` instead of `VertexBuffer` and `IndexBuffer` to simplify APIs. ([#40])
* Rewrited entity component system in a more flexible way. ([#39])

[#39]: https://github.com/shawnscode/crayon/pull/39
[#40]: https://github.com/shawnscode/crayon/pull/40

## 0.1.0 - 2017-12-16
* Initial release

[Unreleased]: https://github.com/shawnscode/crayon/compare/v0.1.0...HEAD