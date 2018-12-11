//! View frustum for visibility determination

use crate::math::prelude::{Aabb3, Plane, PlaneBound, PlaneRelation};

use cgmath::num_traits::cast;
use cgmath::prelude::*;
use cgmath::{BaseFloat, Matrix, Matrix4, Point3, Rad};

/// Projections.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Projection<S: BaseFloat> {
    /// Orthographic projection.
    Ortho {
        /// The width of orthographic window.
        width: S,
        /// The height of orthographic window.
        height: S,
        /// The near clip plane.
        near: S,
        /// The far clip plane.
        far: S,
    },

    /// Perspective projection.
    Perspective {
        /// Field of view in vertical.
        fovy: Rad<S>,
        /// The aspect of width / height.
        aspect: S,
        /// The near clip plane.
        near: S,
        /// The far clip plane.
        far: S,
    },
}

impl<S: BaseFloat> Projection<S> {
    pub fn ortho(width: S, height: S, near: S, far: S) -> Self {
        Projection::Ortho {
            width,
            height,
            near,
            far,
        }
    }

    pub fn perspective(fovy: Rad<S>, aspect: S, near: S, far: S) -> Self {
        Projection::Perspective {
            fovy,
            aspect,
            near,
            far,
        }
    }

    pub fn to_matrix(&self) -> Matrix4<S> {
        Self::matrix(*self)
    }

    pub fn validate(&self) {
        match *self {
            Projection::Perspective {
                fovy,
                aspect,
                near,
                far,
            } => {
                assert!(
                    fovy > Rad::zero(),
                    "The vertical field of view cannot be below zero, found: {:?}",
                    fovy
                );

                assert!(
                    fovy < Rad::turn_div_2(),
                    "The vertical field of view cannot be greater than a half turn, found: {:?}",
                    fovy
                );

                assert!(
                    aspect > S::zero(),
                    "The aspect ratio cannot be below zero, found: {:?}",
                    aspect
                );

                assert!(
                    near > S::zero(),
                    "The near plane distance cannot be below zero, found: {:?}",
                    near
                );

                assert!(
                    far > S::zero(),
                    "The far plane distance cannot be below zero, found: {:?}",
                    far
                );

                assert!(
                    far > near,
                    "The far plane cannot be closer than the near plane, found: far: {:?}, near: {:?}",
                    far,
                    near
                );
            }
            Projection::Ortho {
                width,
                height,
                near,
                far,
            } => {
                assert!(
                    width > S::zero(),
                    "The width cannot be below zero, found: {:?}",
                    width
                );

                assert!(
                    height > S::zero(),
                    "The height cannot be below zero, found: {:?}",
                    height
                );

                assert!(
                    far > near,
                    "The far plane cannot be closer than the near plane, found: far: {:?}, near: {:?}",
                    far,
                    near
                );
            }
        }
    }

    /// Gets the projection matrix in left hand coordinates.
    pub fn matrix(projection: Projection<S>) -> Matrix4<S> {
        match projection {
            Projection::Ortho {
                width,
                height,
                near,
                far,
            } => Self::ortho_matrix(width, height, near, far),
            Projection::Perspective {
                fovy,
                aspect,
                near,
                far,
            } => Self::perspective_matrix(fovy, aspect, near, far),
        }
    }

    /// Gets the orthographic projection matrix in left hand coordinates.
    pub fn ortho_matrix(w: S, h: S, n: S, f: S) -> Matrix4<S> {
        let half: S = cast(0.5).unwrap();
        let two: S = cast(2.0).unwrap();
        let zero = S::zero();
        let one = S::one();

        let (hw, hh) = (w * half, h * half);

        let (l0, r0) = (-hw, hw);
        let (b0, t0) = (-hh, hh);

        let c0 = [two / (r0 - l0), zero, zero, zero];
        let c1 = [zero, two / (t0 - b0), zero, zero];
        let c2 = [zero, zero, two / (f - n), zero];
        let c3 = [
            (r0 + l0) / (l0 - r0),
            (t0 + b0) / (b0 - t0),
            (f + n) / (n - f),
            one,
        ];
        Matrix4::from_cols(c0.into(), c1.into(), c2.into(), c3.into())
    }

    /// Gets the perspective projection matrix in left hand coordinates.
    pub fn perspective_matrix(fovy: Rad<S>, aspect: S, n: S, f: S) -> Matrix4<S> {
        let half: S = cast(0.5).unwrap();
        let two: S = cast(2.0).unwrap();
        let zero = S::zero();
        let one = S::one();

        let fc = Rad::cot(fovy * half);
        let c0 = [fc / aspect, zero, zero, zero];
        let c1 = [zero, fc, zero, zero];
        let c2 = [zero, zero, (f + n) / (f - n), one];
        let c3 = [zero, zero, (two * f * n) / (n - f), zero];
        Matrix4::from_cols(c0.into(), c1.into(), c2.into(), c3.into())
    }
}

/// View frustum, used for frustum culling
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Frustum<S: BaseFloat> {
    projection: Projection<S>,

    /// Left plane
    pub left: Plane<S>,
    /// Right plane
    pub right: Plane<S>,
    /// Bottom plane
    pub bottom: Plane<S>,
    /// Top plane
    pub top: Plane<S>,
    /// Near plane
    pub near: Plane<S>,
    /// Far plane
    pub far: Plane<S>,
}

impl<S: BaseFloat> Frustum<S> {
    /// Construct a frustum.
    pub fn new(projection: Projection<S>) -> Frustum<S> {
        projection.validate();
        let mat = Projection::matrix(projection);

        Frustum {
            projection,

            left: Plane::from_vector4_alt(mat.row(3) + mat.row(0))
                .normalize()
                .unwrap(),

            right: Plane::from_vector4_alt(mat.row(3) - mat.row(0))
                .normalize()
                .unwrap(),

            bottom: Plane::from_vector4_alt(mat.row(3) + mat.row(1))
                .normalize()
                .unwrap(),

            top: Plane::from_vector4_alt(mat.row(3) - mat.row(1))
                .normalize()
                .unwrap(),

            near: Plane::from_vector4_alt(mat.row(3) + mat.row(2))
                .normalize()
                .unwrap(),

            far: Plane::from_vector4_alt(mat.row(3) - mat.row(2))
                .normalize()
                .unwrap(),
        }
    }

    pub fn projection(&self) -> Projection<S> {
        self.projection
    }

    /// Find the spatial relation of a bound inside this frustum.
    pub fn contains<B: PlaneBound<S>>(&self, bound: &B) -> PlaneRelation {
        [
            self.left,
            self.right,
            self.top,
            self.bottom,
            self.near,
            self.far,
        ]
            .iter()
            .fold(PlaneRelation::In, |cur, p| {
                use std::cmp::max;
                let r = bound.relate(*p);
                // If any of the planes are `Out`, the bound is outside.
                // Otherwise, if any are `Cross`, the bound is crossing.
                // Otherwise, the bound is fully inside.
                max(cur, r)
            })
    }

    pub fn to_matrix(&self) -> Matrix4<S> {
        self.projection.to_matrix()
    }
}

/// View frustum corner points
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct FrustumPoints<S: BaseFloat> {
    /// Near top left point
    pub near_top_left: Point3<S>,
    /// Near top right point
    pub near_top_right: Point3<S>,
    /// Near bottom left point
    pub near_bottom_left: Point3<S>,
    /// Near bottom right point
    pub near_bottom_right: Point3<S>,
    /// Far top left point
    pub far_top_left: Point3<S>,
    /// Far top right point
    pub far_top_right: Point3<S>,
    /// Far bottom left point
    pub far_bottom_left: Point3<S>,
    /// Far bottom right point
    pub far_bottom_right: Point3<S>,
}

impl<S: BaseFloat> FrustumPoints<S> {
    /// Apply an arbitrary transform to the corners of this frustum, return a new
    /// conservative frustum.
    #[inline]
    pub fn transform<T>(&self, transform: &T) -> Self
    where
        T: Transform<Point3<S>>,
    {
        FrustumPoints {
            near_top_left: transform.transform_point(self.near_top_left),
            near_top_right: transform.transform_point(self.near_top_right),
            near_bottom_left: transform.transform_point(self.near_bottom_left),
            near_bottom_right: transform.transform_point(self.near_bottom_right),
            far_top_left: transform.transform_point(self.far_top_left),
            far_top_right: transform.transform_point(self.far_top_right),
            far_bottom_left: transform.transform_point(self.far_bottom_left),
            far_bottom_right: transform.transform_point(self.far_bottom_right),
        }
    }

    /// Compute corners.
    #[inline]
    pub fn to_corners(&self) -> [Point3<S>; 8] {
        [
            self.near_top_left,
            self.near_top_right,
            self.near_bottom_left,
            self.near_bottom_right,
            self.far_top_left,
            self.far_top_right,
            self.far_bottom_left,
            self.far_bottom_right,
        ]
    }

    /// Compute aabb.
    #[inline]
    pub fn aabb(&self) -> Aabb3<S> {
        let aabb = Aabb3::zero();
        self.to_corners()[..]
            .iter()
            .fold(aabb, |u, &corner| u.grow(corner))
    }
}

impl<S: BaseFloat> Into<FrustumPoints<S>> for Frustum<S> {
    fn into(self) -> FrustumPoints<S> {
        match self.projection {
            Projection::Ortho {
                width,
                height,
                near,
                far,
            } => {
                let half: S = cast(0.5).unwrap();
                let (hw, hh) = (width * half, height * half);
                let (l, r) = (-hw, hw);
                let (b, t) = (-hh, hh);

                FrustumPoints {
                    near_top_left: Point3::new(near, t, l),
                    near_top_right: Point3::new(near, t, r),
                    near_bottom_left: Point3::new(near, b, l),
                    near_bottom_right: Point3::new(near, b, r),
                    far_top_left: Point3::new(far, t, l),
                    far_top_right: Point3::new(far, t, r),
                    far_bottom_left: Point3::new(far, b, l),
                    far_bottom_right: Point3::new(far, b, r),
                }
            }
            Projection::Perspective {
                fovy,
                aspect,
                near,
                far,
            } => {
                let m = Projection::perspective_matrix(fovy, aspect, near, far);
                let im = m.invert().unwrap();
                let one = S::one();

                let points = FrustumPoints {
                    near_top_left: Point3::new(-one, one, -one),
                    near_top_right: Point3::new(-one, one, one),
                    near_bottom_left: Point3::new(-one, -one, -one),
                    near_bottom_right: Point3::new(-one, -one, one),
                    far_top_left: Point3::new(one, one, -one),
                    far_top_right: Point3::new(one, one, one),
                    far_bottom_left: Point3::new(one, -one, -one),
                    far_bottom_right: Point3::new(one, -one, one),
                };

                points.transform(&im)
            }
        }
    }
}

impl<S: BaseFloat> Into<Matrix4<S>> for Frustum<S> {
    fn into(self) -> Matrix4<S> {
        self.projection.to_matrix()
    }
}
