use crayon::math::prelude::*;

/// `Transform` is used to store and manipulate the postiion, rotation and scale
/// of the object. We use a left handed, y-up world coordinate system.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Transform {
    matrix: Matrix2<f32>,
    disp: Vector2<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            matrix: Matrix2::one(),
            disp: Vector2::zero(),
        }
    }
}

impl ::std::ops::Mul for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let m = self.matrix * rhs.matrix;

        let t = Vector2::new(
            self.matrix[0][0] * rhs.disp[0] + self.matrix[0][1] * rhs.disp[1] + self.disp.x,
            self.matrix[1][0] * rhs.disp[0] + self.matrix[1][1] * rhs.disp[1] + self.disp.y,
        );

        Transform { matrix: m, disp: t }
    }
}

impl Transform {
    /// Create a rotation transform by a given angle.
    #[inline]
    pub fn rotation<T: Into<Rad<f32>>>(theta: T) -> Self {
        Transform {
            matrix: Matrix2::from_angle(theta),
            disp: Vector2::zero(),
        }
    }

    /// Creates a scale transform.
    #[inline]
    pub fn scale<T: Into<Vector2<f32>>>(scale: T) -> Self {
        let s = scale.into();
        Transform {
            matrix: Matrix2::new(s[0], 0.0, 0.0, s[1]),
            disp: Vector2::zero(),
        }
    }

    /// Creates a translation transform.
    #[inline]
    pub fn translation<T: Into<Vector2<f32>>>(disp: T) -> Self {
        Transform {
            matrix: Matrix2::one(),
            disp: disp.into(),
        }
    }

    /// Creates a shear transform.
    #[inline]
    pub fn shear<T: Into<Vector2<f32>>>(skew: T) -> Self {
        let s = skew.into();
        Transform {
            matrix: Matrix2::new(0.0, s[0], s[1], 0.0),
            disp: Vector2::zero(),
        }
    }
}

impl Transform {
    /// Returns a transform that "un-does" this one.
    #[inline]
    pub fn inverse(self) -> Option<Self> {
        let dt = self.matrix.determinant();

        if dt <= ::std::f32::EPSILON {
            None
        } else {
            let inverse_matrix = Matrix2::new(
                self.matrix[1][1] / dt,
                -self.matrix[0][1] / dt,
                -self.matrix[1][0] / dt,
                self.matrix[0][0] / dt,
            );

            let inverse_disp = Vector2::new(
                (self.matrix[1][0] * self.disp[1] - self.matrix[1][1] * self.disp[0]) / dt,
                (self.matrix[0][0] * self.disp[1] - self.matrix[0][1] * self.disp[0]) / dt,
            );

            Some(Transform {
                matrix: inverse_matrix,
                disp: inverse_disp,
            })
        }
    }

    /// Transforms direction from local space to transform's space.
    ///
    /// This operation is not affected by scale or position of the transform. The returned
    /// vector has the same length as direction.
    #[inline]
    pub fn transform_direction<T>(&self, v: T) -> Vector2<f32>
    where
        T: Into<Vector2<f32>>,
    {
        (self.matrix * v.into()).normalize()
    }

    /// Transforms vector from local space to transform's space.
    ///
    /// This operation is not affected by position of the transform, but is is affected by scale.
    /// The returned vector may have a different length than vector.
    #[inline]
    pub fn transform_vector<T>(&self, v: T) -> Vector2<f32>
    where
        T: Into<Vector2<f32>>,
    {
        self.matrix * v.into()
    }

    /// Transforms points from local space to transform's space.
    #[inline]
    pub fn transform_point<T>(&self, v: T) -> Vector2<f32>
    where
        T: Into<Vector2<f32>>,
    {
        self.matrix * v.into() + self.disp
    }

    /// Returns the up direction in transform's space, which is looking down the positive y-axis.
    #[inline]
    pub fn up(&self) -> Vector2<f32> {
        self.transform_direction(Vector2::new(0.0, 1.0))
    }

    /// Returns the right direction in transform's space, which is looking down the positive x-axis.
    #[inline]
    pub fn right(&self) -> Vector2<f32> {
        self.transform_direction(Vector2::new(1.0, 0.0))
    }
}
