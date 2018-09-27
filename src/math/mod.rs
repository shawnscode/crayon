//! This module contains the math utils that mainly comes from `cgmath` and `collision-rs`.

pub mod aabb;
pub mod color;
pub mod frustum;
pub mod plane;

pub mod prelude {
    pub use super::aabb::{Aabb2, Aabb3};
    pub use super::color::Color;
    pub use super::frustum::{Frustum, FrustumPoints, Projection};
    pub use super::plane::{Plane, PlaneBound, PlaneRelation};

    pub use cgmath::prelude::{EuclideanSpace, InnerSpace, MetricSpace, VectorSpace};
    pub use cgmath::prelude::{One, Zero};
    pub use cgmath::{Angle, Deg, Euler, Quaternion, Rad, Rotation};
    pub use cgmath::{Matrix, Matrix2, Matrix3, Matrix4, SquareMatrix, Vector2, Vector3, Vector4};
}
