//! The entity component system with a data-orinted designs.
//!
//! # Entity Component System (ECS)
//!
//! ECS is an architectural pattern that is widely used in game development. It follows
//! the _Composition_ over _Inheritance_ principle that allows greater flexibility in
//! defining entities where every object in a game's scene in an entity.
//!
//! ## Entity
//!
//! `Entity` is one of the most fundamental terms in this system. Its basicly some kind
//! of unique identifier to the in-game object. Every `Entity`s comes with one or more
//! `Component`s, which define the internal data and how it interacts with the world.
//!
//! Its also common that abstracts `Entity` as container of components, buts with UID
//! approach, we could save the state externaly, users could transfer `Entity` easily
//! without considering the data-ownerships. The real data storage can be shuffled around
//! in memory as needed.
//!
//! ```rust
//! use crayon::ecs::prelude::*;
//! let mut world = World::new();
//!
//! // Creates a new and empty `Entity`.
//! let e1 = world.create();
//!
//! // Recycles the `Entity` handle, and frees corresponding components.
//! world.free(e1);
//! ```
//!
//! ## Component
//!
//! `Component` is the raw data for one aspect of the object, and how it interacts with
//! the world. Every kind of `Component`s is stored in some kind of storage arena, and
//! need to be specified during implementation.
//!
//! ### Data Orinted Design
//!
//! Data-oriented design is a program optimization approach motivated by cache coherency.
//! The approach is to focus on the data layout, separating and sorting fields according
//! to when they are needed, and to think about transformations of data.
//!
//! Due to the composition nature of ECS, its highly compatible with DOD. But benefits
//! doesn't comes for free, there are some memory/performance tradeoff generally. We
//! addressed some data storage approaches in `ecs::component`, users could make their
//! own decision based on different purposes.
//!
//! ### View
//!
//! Its common to iterate all the entities that consists of some specific `Component`s. We
//! addresses a `Join` trait to provide a way to access entities and its components at same
//! time.
//!
//!
//! ```rust
//! use crayon::ecs::prelude::*;
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct Position(i32, i32, i32);
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct Label(String);
//!
//! // Vec based storage, supposed to have maximum performance for the components
//! // mostly present in entities.
//! impl Component for Position {
//!     type Arena = VecArena<Position>;
//! }
//!
//! // HashMap based storage which are best suited for rare components.
//! impl Component for Label {
//!     type Arena = HashMapArena<Label>;
//! }
//!
//! let mut world = World::new();
//!
//! // Components must be registered before using in `World`.
//! world.register::<Position>();
//! world.register::<Label>();
//!
//! let e1 = world.create();
//! world.add(e1, Position(0, 1, 0));
//! world.add(e1, Label(String::from("Player")));
//!
//! let e2 = world.create();
//! world.add(e2, Position(0, 0, 0));
//! world.add(e2, Label(String::from("Enemy")));
//!
//! let e3 = world.create();
//! world.add(e3, Label(String::from("Enemy")));
//!
//! {
//!     // Gets the reference to the component from `World` directly.
//!     assert_eq!(*world.get::<Position>(e1).unwrap(), Position(0, 1, 0));
//!
//!     // Immutably borrow the underlying storage of `Position`.
//!     let (_, positions) = world.view_r1::<Position>();
//!     // Gets the reference to the component from `Arena`.
//!     assert_eq!(*positions.get(e1).unwrap(), Position(0, 1, 0));
//! }
//!
//! {
//!     let (entities, labels, mut positions) = world.view_r1w1::<Label, Position>();
//!
//!     /// We can get entities which have both of `Position` and `Label` by joining.
//!     for (label, mut position) in (&labels, &mut positions).components(&entities) {
//!         if label.0 == "Enemy" {
//!             position.0 += 1;
//!         }
//!     }
//! }
//!
//! // The `Enemy`(e2) has been advance one step in x-axis.
//! assert_eq!(world.get::<Position>(e1).unwrap().0, 0);
//! assert_eq!(world.get::<Position>(e2).unwrap().0, 1);
//! assert_eq!(world.get::<Position>(e3), None);
//! ```
//!
//! ## System and Dispatcher
//!
//! __TODO__

pub mod bitset;
pub mod component;
pub mod world;
pub mod system;
pub mod view;

pub mod prelude {
    pub use super::component::{Component, HashMapArena, VecArena};
    pub use super::world::{Entity, EntityBuilder, World};
    // pub use super::system::{System, SystemMut};
    pub use super::view::{ArenaGet, ArenaGetMut, Fetch, FetchMut, Join};
}
