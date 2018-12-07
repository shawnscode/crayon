use std::fmt;

use cgmath::prelude::*;
use cgmath::{BaseFloat, Point3, Vector3, Vector4};

/// A 3-dimensional plane formed from the equation: `A*x + B*y + C*z - D = 0`.
///
/// # Fields
///
/// - `n`: a unit vector representing the normal of the plane where:
///   - `n.x`: corresponds to `A` in the plane equation
///   - `n.y`: corresponds to `B` in the plane equation
///   - `n.z`: corresponds to `C` in the plane equation
/// - `d`: the distance value, corresponding to `D` in the plane equation
///
/// # Notes
///
/// The `A*x + B*y + C*z - D = 0` form is preferred over the other common
/// alternative, `A*x + B*y + C*z + D = 0`, because it tends to avoid
/// superfluous negations (see _Real Time Collision Detection_, p. 55).
#[derive(Copy, Clone, PartialEq)]
pub struct Plane<S> {
    /// Plane normal
    pub n: Vector3<S>,
    /// Plane distance value
    pub d: S,
}

impl<S: BaseFloat> Plane<S> {
    /// Construct a plane from a normal vector and a scalar distance. The
    /// plane will be perpendicular to `n`, and `d` units offset from the
    /// origin.
    pub fn new(n: Vector3<S>, d: S) -> Plane<S> {
        Plane { n, d }
    }

    /// # Arguments
    ///
    /// - `a`: the `x` component of the normal
    /// - `b`: the `y` component of the normal
    /// - `c`: the `z` component of the normal
    /// - `d`: the plane's distance value
    pub fn from_abcd(a: S, b: S, c: S, d: S) -> Plane<S> {
        Plane {
            n: Vector3::new(a, b, c),
            d,
        }
    }

    /// Construct a plane from the components of a four-dimensional vector
    pub fn from_vector4(v: Vector4<S>) -> Plane<S> {
        Plane {
            n: Vector3::new(v.x, v.y, v.z),
            d: v.w,
        }
    }

    /// Construct a plane from the components of a four-dimensional vector
    /// Assuming alternative representation: `A*x + B*y + C*z + D = 0`
    pub fn from_vector4_alt(v: Vector4<S>) -> Plane<S> {
        Plane {
            n: Vector3::new(v.x, v.y, v.z),
            d: -v.w,
        }
    }

    /// Constructs a plane that passes through the the three points `a`, `b` and `c`
    pub fn from_points(a: Point3<S>, b: Point3<S>, c: Point3<S>) -> Option<Plane<S>> {
        // create two vectors that run parallel to the plane
        let v0 = b - a;
        let v1 = c - a;

        // find the normal vector that is perpendicular to v1 and v2
        let n = v0.cross(v1);

        if ulps_eq!(n, &Vector3::zero()) {
            None
        } else {
            // compute the normal and the distance to the plane
            let n = n.normalize();
            let dis = -a.dot(n);

            Some(Plane::new(n, dis))
        }
    }

    /// Construct a plane from a point and a normal vector.
    /// The plane will contain the point `p` and be perpendicular to `n`.
    pub fn from_point_normal(p: Point3<S>, n: Vector3<S>) -> Plane<S> {
        Plane { n, d: p.dot(n) }
    }

    /// Normalize a plane.
    pub fn normalize(&self) -> Option<Plane<S>> {
        if ulps_eq!(self.n, &Vector3::zero()) {
            None
        } else {
            let denom = S::one() / self.n.magnitude();
            Some(Plane::new(self.n * denom, self.d * denom))
        }
    }
}

impl<S: BaseFloat> fmt::Debug for Plane<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?}x + {:?}y + {:?}z - {:?} = 0",
            self.n.x, self.n.y, self.n.z, self.d
        )
    }
}

/// Spatial relation between two objects.
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum PlaneRelation {
    /// Completely inside.
    In,
    /// Crosses the boundary.
    Cross,
    /// Completely outside.
    Out,
}

/// Generic 3D bound.
pub trait PlaneBound<S: BaseFloat>: fmt::Debug {
    /// Classify the spatial relation with a plane.
    fn relate(&self, plane: Plane<S>) -> PlaneRelation;
}

impl<S: BaseFloat> PlaneBound<S> for Point3<S> {
    fn relate(&self, plane: Plane<S>) -> PlaneRelation {
        let dist = self.dot(plane.n);
        if dist > plane.d {
            PlaneRelation::In
        } else if dist < plane.d {
            PlaneRelation::Out
        } else {
            PlaneRelation::Cross
        }
    }
}
