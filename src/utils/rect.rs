use math;

/// A rectangle, with top-left corner at `min`, and bottom-right corner at `max`.
#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub min: math::Point2<i32>,
    pub max: math::Point2<i32>,
}

impl Rect {
    #[inline]
    pub fn new(min: math::Point2<i32>, max: math::Point2<i32>) -> Self {
        Rect { min: min, max: max }
    }

    #[inline]
    pub fn width(&self) -> i32 {
        self.max.x - self.min.x
    }

    #[inline]
    pub fn height(&self) -> i32 {
        self.max.y - self.min.y
    }

    #[inline]
    pub fn size(&self) -> i32 {
        self.width() * self.height()
    }

    #[inline]
    pub fn overlap(&self, rhs: Self) -> Self {
        use std::cmp;
        Rect {
            min: math::Point2::new(cmp::max(self.min.x, rhs.min.x),
                                   cmp::max(self.min.y, rhs.min.y)),
            max: math::Point2::new(cmp::min(self.max.x, rhs.max.x),
                                   cmp::min(self.max.y, rhs.max.y)),
        }
    }

    #[inline]
    pub fn contains<P>(&self, p: P) -> bool
        where P: Into<math::Point2<i32>>
    {
        let p = p.into();
        p.x >= self.min.x && p.x < self.max.x && p.y >= self.min.y && p.y < self.max.y
    }
}
