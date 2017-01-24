#### Build Status
[![Build Status](https://travis-ci.org/kayak233/lemon3d.svg?branch=master)](https://travis-ci.org/kayak233/lemon3d)

#### Introduction
Lemon3d is an experimental purpose project, written with a minimalistic modular design philosophy. Its built from the ground up to focus on cache friendly data layouts in multicore environments with entity-component based architecture.

*Warning*: Its far from a stable version right now.

#### Features
- [x] \[ECS\] Entity component system with a data-driven designs.
- [x] \[TSK\] Task based multi-thread system based on awesome crate [Rayon](https://github.com/nikomatsakis/rayon.git).

#### Roadmap v0.0.2 (Basic Usages)

##### Graphics Subsystem
- [] Window and graphic context management based on SDL2.
- [] Stateless, layered, multi-threaded graphics subsystem based on OpenGL.
- [] Graphics resource management.
- [] TrueType font integrations.
- [] Vector drawing library for ui and visualizations.
- [] Bloat-free immediate mode graphics user interface.

##### Scene Subsystem
- [] Hierachy-based transformation of position/rotaion/scale etc.
- [] 2D Layout facilities like anchor, pivot in Unity3D.
- [] Easy tween and action facilicities.

##### Resource Subsystem
- [] Abstract archive with default supports for native filesystem and zip package.
- [] Assets load and cache machanism based on LRU(maybe).
- [] Serilization/deserlization of entities and components in YAML or binary mode.

##### Script Subsystem
- [] High-level Lua 5.3 integration to Rust.
- [] Exports Rust codes to lua with macros.

##### Tools
- [] Command-line interface for creating and deploying game projects.

#### FAQ
**Why Rust ?**
First of all, this is a part-time toy project of myself,  so i don't really care if we ever have a game engine written in Rust.

And the most importantly, it makes sense for me to take the numerous advantages of modern programming language, instead of using languages like C/C++ which have many historical burden. In fact, this project has beed development with C++ for about four months, which produce a basic multi-thread, and much more pains.