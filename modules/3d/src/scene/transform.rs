use crayon::math::{self, One};

/// `Transform` is used to store and manipulate the postiion, rotation and scale
/// of the object. We use a left handed, y-up world coordinate system.
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub scale: f32,
    pub position: math::Vector3<f32>,
    pub rotation: math::Quaternion<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            scale: 1.0,
            position: math::Vector3::new(0.0, 0.0, 0.0),
            rotation: math::Quaternion::one(),
        }
    }
}

impl ::std::ops::Mul for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Transform {
            position: rhs.position + self.position,
            rotation: rhs.rotation * self.rotation,
            scale: rhs.scale * self.scale,
        }
    }
}

impl Transform {
    /// Transforms direction from local space to transform's space.
    ///
    /// This operation is not affected by scale or position of the transform. The returned
    /// vector has the same length as direction.
    #[inline]
    pub fn transform_direction<T>(&self, v: T) -> math::Vector3<f32>
    where
        T: Into<math::Vector3<f32>>,
    {
        self.rotation * v.into()
    }

    /// Transforms vector from local space to transform's space.
    ///
    /// This operation is not affected by position of the transform, but is is affected by scale.
    /// The returned vector may have a different length than vector.
    #[inline]
    pub fn transform_vector<T>(&self, v: T) -> math::Vector3<f32>
    where
        T: Into<math::Vector3<f32>>,
    {
        self.rotation * (v.into() * self.scale)
    }

    /// Transforms points from local space to transform's space.
    #[inline]
    pub fn transform_point<T>(&self, v: T) -> math::Vector3<f32>
    where
        T: Into<math::Vector3<f32>>,
    {
        self.rotation * (v.into() * self.scale) + self.position
    }

    /// Returns the up direction in transform's space, which is looking down the positive y-axis.
    #[inline]
    pub fn up(&self) -> math::Vector3<f32> {
        self.transform_direction(math::Vector3::new(0.0, 1.0, 0.0))
    }

    /// Returns the forward direction in transform's space, which is looking down the positive z-axis.
    #[inline]
    pub fn forward(&self) -> math::Vector3<f32> {
        self.transform_direction(math::Vector3::new(0.0, 0.0, 1.0))
    }

    /// Returns the right direction in transform's space, which is looking down the positive x-axis.
    #[inline]
    pub fn right(&self) -> math::Vector3<f32> {
        self.transform_direction(math::Vector3::new(1.0, 0.0, 0.0))
    }
}
