//! This module contains the math utils that mainly comes from `cgmath` and `collision-rs`.

pub use cgmath::*;

pub mod plane;
pub use self::plane::{Plane, PlaneBound, PlaneRelation};

pub mod aabb;
pub use self::aabb::{Aabb2, Aabb3};

pub mod frustum;
pub use self::frustum::{Frustum, FrustumPoints, Projection};
