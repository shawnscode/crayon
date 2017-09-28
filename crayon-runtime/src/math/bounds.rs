use super::*;

/// `Bounds` represents an axis aligned bounding box.
///
/// An axis-aligned bounding box, or AABB for short, is a box aligned with
/// coordinate axes and fully enclosing some object. Because the box is
/// never rotated with respect to the axes, it can be defined by just its
/// center and extents, or alternatively by min and max points.
pub struct Bounds {
    /// The center of the bounding box.
    pub pos: Vector3<f32>,
    /// The extents of the box. This is always half of the size.
    pub extents: Vector3<f32>,
}

impl Bounds {
    /// Create a new bounds with minimal and maximal value.
    pub fn with_bounds(min: Vector3<f32>, max: Vector3<f32>) -> Bounds {
        Bounds {
            pos: (max + min) * 0.5,
            extents: (max - min) * 0.5,
        }

    }

    /// The maximal point of the box. This is always equal to center+extents.
    #[inline]
    pub fn max(&self) -> Vector3<f32> {
        self.pos + self.extents
    }

    /// The minimal point of the box. This is always equal to center-extents.
    #[inline]
    pub fn min(&self) -> Vector3<f32> {
        self.pos - self.extents
    }

    /// Return true if the point passed into `contains` is inside the bounding box.
    #[inline]
    pub fn contains(&self, v: Vector3<f32>) -> bool {
        let min = self.min();
        let max = self.max();
        v.x >= min.x && v.x < max.x && v.y >= min.y && v.y < max.y && v.z >= min.z && v.z < max.z
    }

    /// Merge two bounds into one.
    pub fn merge(&self, rhs: Bounds) -> Bounds {
        let lhs_min = self.min();
        let lhs_max = self.max();

        let rhs_min = rhs.min();
        let rhs_max = rhs.max();

        let new_min = Vector3::new(lhs_min.x.min(rhs_min.x),
                                   lhs_min.y.min(rhs_min.y),
                                   lhs_min.z.min(rhs_min.z));

        let new_max = Vector3::new(lhs_max.x.max(rhs_max.x),
                                   lhs_max.y.max(rhs_max.y),
                                   lhs_max.z.max(rhs_max.z));

        Bounds::with_bounds(new_min, new_max)
    }
}
