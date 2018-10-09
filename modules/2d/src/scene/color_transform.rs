use crayon::math::prelude::*;

/// The `ColorTransform` defines a simple transform that can be applied to
/// the color space of a graphic object. The following are the two types
/// of transform possible:
///
/// 1. multiplication transforms
/// 2. addition transforms
///
/// Addition and multiplication transforms can be combined as follows:
/// R' = max(0, min(R * red_mult_term) + red_add_term,    255))
/// G' = max(0, min(G * green_mult_term + green_add_term,  255))
/// B' = max(0, min(B * blue_mult_term + blue_add_term,   255))
/// A' = max(0, min(A * alpha_mult_term + alpha_add_term,  255))
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ColorTransform {
    pub mul: Vector4<f32>,
    pub add: Vector4<f32>,
}

impl Default for ColorTransform {
    fn default() -> Self {
        ColorTransform {
            mul: Vector4::new(1.0, 1.0, 1.0, 1.0),
            add: Vector4::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}

impl ::std::ops::Mul for ColorTransform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        ColorTransform {
            mul: Vector4::new(
                self.mul[0] * rhs.mul[0],
                self.mul[1] * rhs.mul[1],
                self.mul[2] * rhs.mul[2],
                self.mul[3] * rhs.mul[3],
            ),
            add: self.add + rhs.add,
        }
    }
}

impl ColorTransform {
    /// Returns a transform that "un-does" this one.
    #[inline]
    pub fn inverse(self) -> Option<Self> {
        if self.mul[0].abs() <= ::std::f32::EPSILON
            || self.mul[1].abs() <= ::std::f32::EPSILON
            || self.mul[2].abs() <= ::std::f32::EPSILON
            || self.mul[3].abs() <= ::std::f32::EPSILON
        {
            None
        } else {
            let inv_mul = Vector4::new(
                1.0 / self.mul[0],
                1.0 / self.mul[1],
                1.0 / self.mul[2],
                1.0 / self.mul[4],
            );

            Some(ColorTransform {
                mul: inv_mul,
                add: -self.add,
            })
        }
    }

    /// Transforms color.
    #[inline]
    pub fn transform<T: Into<Color<f32>>>(&self, color: T) -> Color<f32> {
        let c = color.into();
        Color::new(
            (self.mul[0] * c.r + self.add[0]).max(0.0).min(1.0),
            (self.mul[1] * c.g + self.add[1]).max(0.0).min(1.0),
            (self.mul[2] * c.b + self.add[2]).max(0.0).min(1.0),
            (self.mul[3] * c.a + self.add[3]).max(0.0).min(1.0),
        )
    }
}
