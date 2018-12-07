//! Axis-aligned bounding boxes
//!
//! An AABB is a geometric object which encompasses a set of points and is not
//! rotated. It is either a rectangle or a rectangular prism (depending on the
//! dimension) where the slope of every line is either 0 or undefined. These
//! are useful for very cheap collision detection.

use std::cmp::{Ordering, PartialOrd};
use std::fmt;

use cgmath::prelude::*;
use cgmath::{BaseFloat, BaseNum, Point2, Point3, Vector2, Vector3};
use crate::math::prelude::{Plane, PlaneBound, PlaneRelation};

/// A two-dimensional AABB, aka a rectangle.
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq)]
pub struct Aabb2<S> {
    /// Minimum point of the AABB.
    pub min: Point2<S>,
    /// Maximum point of the AABB.
    pub max: Point2<S>,
}

impl<S: BaseNum> Aabb2<S> {
    /// Construct a empty axis-aligned bounding box.
    #[inline]
    pub fn zero() -> Self {
        Aabb2 {
            min: Point2::new(S::zero(), S::zero()),
            max: Point2::new(S::zero(), S::zero()),
        }
    }

    /// Construct a new axis-aligned bounding box from two points.
    #[inline]
    pub fn new(p1: Point2<S>, p2: Point2<S>) -> Aabb2<S> {
        Aabb2 {
            min: Point2::new(min(p1.x, p2.x), min(p1.y, p2.y)),
            max: Point2::new(max(p1.x, p2.x), max(p1.y, p2.y)),
        }
    }

    /// Compute corners.
    #[inline]
    pub fn to_corners(&self) -> [Point2<S>; 4] {
        [
            self.min,
            Point2::new(self.max.x, self.min.y),
            Point2::new(self.min.x, self.max.y),
            self.max,
        ]
    }

    /// Return a shared reference to the point nearest to (-inf, -inf).
    #[inline]
    pub fn min(&self) -> Point2<S> {
        self.min
    }

    /// Return a shared reference to the point nearest to (inf, inf).
    #[inline]
    pub fn max(&self) -> Point2<S> {
        self.max
    }

    /// Return the dimensions of this AABB.
    #[inline]
    pub fn dim(&self) -> Vector2<S> {
        self.max() - self.min()
    }

    /// Return the volume this AABB encloses.
    #[inline]
    pub fn volume(&self) -> S {
        self.dim().product()
    }

    /// Return the center point of this AABB.
    #[inline]
    pub fn center(&self) -> Point2<S> {
        let two = S::one() + S::one();
        self.min() + self.dim() / two
    }

    /// Returns a new AABB that is grown to include the given point.
    #[inline]
    pub fn grow(&self, p: Point2<S>) -> Self {
        Self::new(MinMax::min(self.min(), p), MinMax::max(self.max(), p))
    }

    /// Add a margin of the given width around the AABB, returning a new AABB.
    #[inline]
    pub fn add_margin(&self, margin: Vector2<S>) -> Self {
        Aabb2::new(
            Point2::new(self.min.x - margin.x, self.min.y - margin.y),
            Point2::new(self.max.x + margin.x, self.max.y + margin.y),
        )
    }

    /// Apply an arbitrary transform to the corners of this bounding box,
    /// return a new conservative bound.
    #[inline]
    pub fn transform<T>(&self, transform: &T) -> Self
    where
        T: Transform<Point2<S>>,
    {
        let corners = self.to_corners();
        let transformed_first = transform.transform_point(corners[0]);
        let base = Self::new(transformed_first, transformed_first);
        corners[1..]
            .iter()
            .fold(base, |u, &corner| u.grow(transform.transform_point(corner)))
    }
}

impl<S: BaseNum> fmt::Debug for Aabb2<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:?} - {:?}]", self.min, self.max)
    }
}

/// A three-dimensional AABB, aka a rectangular prism.
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq)]
pub struct Aabb3<S> {
    /// Minimum point of the AABB
    pub min: Point3<S>,
    /// Maximum point of the AABB
    pub max: Point3<S>,
}

impl<S: BaseNum> Aabb3<S> {
    /// Construct a empty axis-aligned bounding box.
    #[inline]
    pub fn zero() -> Self {
        Aabb3 {
            min: Point3::new(S::zero(), S::zero(), S::zero()),
            max: Point3::new(S::zero(), S::zero(), S::zero()),
        }
    }

    /// Construct a new axis-aligned bounding box from two points.
    #[inline]
    pub fn new(p1: Point3<S>, p2: Point3<S>) -> Aabb3<S> {
        Aabb3 {
            min: Point3::new(min(p1.x, p2.x), min(p1.y, p2.y), min(p1.z, p2.z)),
            max: Point3::new(max(p1.x, p2.x), max(p1.y, p2.y), max(p1.z, p2.z)),
        }
    }

    /// Compute corners.
    #[inline]
    pub fn to_corners(&self) -> [Point3<S>; 8] {
        [
            self.min,
            Point3::new(self.max.x, self.min.y, self.min.z),
            Point3::new(self.min.x, self.max.y, self.min.z),
            Point3::new(self.max.x, self.max.y, self.min.z),
            Point3::new(self.min.x, self.min.y, self.max.z),
            Point3::new(self.max.x, self.min.y, self.max.z),
            Point3::new(self.min.x, self.max.y, self.max.z),
            self.max,
        ]
    }

    /// Return a shared reference to the point nearest to (-inf, -inf).
    #[inline]
    pub fn min(&self) -> Point3<S> {
        self.min
    }

    /// Return a shared reference to the point nearest to (inf, inf).
    #[inline]
    pub fn max(&self) -> Point3<S> {
        self.max
    }

    /// Return the dimensions of this AABB.
    #[inline]
    pub fn dim(&self) -> Vector3<S> {
        self.max() - self.min()
    }

    /// Return the volume this AABB encloses.
    #[inline]
    pub fn volume(&self) -> S {
        self.dim().product()
    }

    /// Return the center point of this AABB.
    #[inline]
    pub fn center(&self) -> Point3<S> {
        let two = S::one() + S::one();
        self.min() + self.dim() / two
    }

    /// Returns a new AABB that is grown to include the given point.
    #[inline]
    pub fn grow(&self, p: Point3<S>) -> Self {
        Self::new(MinMax::min(self.min(), p), MinMax::max(self.max(), p))
    }

    /// Add a margin of the given width around the AABB, returning a new AABB.
    #[inline]
    pub fn add_margin(&self, margin: Vector3<S>) -> Self {
        Aabb3::new(
            Point3::new(
                self.min.x - margin.x,
                self.min.y - margin.y,
                self.min.z - margin.z,
            ),
            Point3::new(
                self.max.x + margin.x,
                self.max.y + margin.y,
                self.max.z + margin.z,
            ),
        )
    }

    /// Apply an arbitrary transform to the corners of this bounding box,
    /// return a new conservative bound.
    #[inline]
    pub fn transform<T>(&self, transform: &T) -> Self
    where
        T: Transform<Point3<S>>,
    {
        let corners = self.to_corners();
        let transformed_first = transform.transform_point(corners[0]);
        let base = Self::new(transformed_first, transformed_first);
        corners[1..]
            .iter()
            .fold(base, |u, &corner| u.grow(transform.transform_point(corner)))
    }
}

impl<S: BaseNum> fmt::Debug for Aabb3<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{:?} - {:?}]", self.min, self.max)
    }
}

impl<S: BaseFloat> PlaneBound<S> for Aabb3<S> {
    fn relate(&self, plane: Plane<S>) -> PlaneRelation {
        let corners = self.to_corners();
        let first = corners[0].relate(plane);
        for p in corners[1..].iter() {
            if p.relate(plane) != first {
                return PlaneRelation::Cross;
            }
        }
        first
    }
}

fn min<S: PartialOrd + Copy>(lhs: S, rhs: S) -> S {
    match lhs.partial_cmp(&rhs) {
        Some(Ordering::Less) | Some(Ordering::Equal) | None => lhs,
        _ => rhs,
    }
}

fn max<S: PartialOrd + Copy>(lhs: S, rhs: S) -> S {
    match lhs.partial_cmp(&rhs) {
        Some(Ordering::Greater) | Some(Ordering::Equal) | None => lhs,
        _ => rhs,
    }
}

/// Compute the minimum/maximum of the given values
pub trait MinMax {
    /// Compute the minimum
    fn min(a: Self, b: Self) -> Self;

    /// Compute the maximum
    fn max(a: Self, b: Self) -> Self;
}

impl<S: PartialOrd> MinMax for Point2<S>
where
    S: BaseNum,
{
    fn min(a: Point2<S>, b: Point2<S>) -> Point2<S> {
        Point2::new(min(a.x, b.x), min(a.y, b.y))
    }

    fn max(a: Point2<S>, b: Point2<S>) -> Point2<S> {
        Point2::new(max(a.x, b.x), max(a.y, b.y))
    }
}

impl<S: PartialOrd> MinMax for Point3<S>
where
    S: BaseNum,
{
    fn min(a: Point3<S>, b: Point3<S>) -> Point3<S> {
        Point3::new(min(a.x, b.x), min(a.y, b.y), min(a.z, b.z))
    }

    fn max(a: Point3<S>, b: Point3<S>) -> Point3<S> {
        Point3::new(max(a.x, b.x), max(a.y, b.y), max(a.z, b.z))
    }
}
