# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog][kc], and this project adheres to
[Semantic Versioning][sv].

[kc]: http://keepachangelog.com/
[sv]: http://semver.org/

## [Unreleased]

## [0.2.1] - 2018-02-02

### Added
* Add headless mode which makes example integration possible. ([#45])

### Fixed
* Fixed clippy linter warnings. ([#46])
* Fixed `Location/LocationAtom::is_shared()`. ([#46])

[#46]: https://github.com/shawnscode/crayon/pull/46
[#45]: https://github.com/shawnscode/crayon/pull/45

## [0.2.0] - 2018-01-30

### Added
* Introduced scene in core module. ([#42][#43])
* Added touch emulation with mouse device. ([#42])

### Fixed
* Fixed unexcepted panics when closing window. ([#40])
* Fixed unexpected `ColorMask` and `DepthMask` behaviours. ([#42])

### Changed
* Introduced `Mesh` instead of `VertexBuffer` and `IndexBuffer` to simplify APIs. ([#40][#41])
* Rewrited entity component system in a more flexible way. ([#39])
* Removed inexplicit location definitions when creating `Shader`, `Texture` and `Mesh` objects. ([#42])

[#39]: https://github.com/shawnscode/crayon/pull/39
[#40]: https://github.com/shawnscode/crayon/pull/40
[#41]: https://github.com/shawnscode/crayon/pull/41
[#42]: https://github.com/shawnscode/crayon/pull/42
[#43]: https://github.com/shawnscode/crayon/pull/43

## 0.1.0 - 2017-12-16
* Initial release

[0.2.0]: https://github.com/shawnscode/crayon/compare/v0.1.0...v0.2.0
[0.2.1]: https://github.com/shawnscode/crayon/compare/v0.1.0...v0.2.1
[Unreleased]: https://github.com/shawnscode/crayon/compare/v0.2.1...HEAD