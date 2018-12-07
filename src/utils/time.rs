use std::time::Duration;

/// A measurement of a monotonically nondecreasing clock.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(u64);

impl Timestamp {
    #[inline]
    pub fn from_millis(millis: u64) -> Timestamp {
        Timestamp(millis)
    }

    #[inline]
    pub fn now() -> Timestamp {
        crate::application::sys::timestamp()
    }

    #[inline]
    pub fn elapsed(self) -> Duration {
        crate::application::sys::timestamp() - self
    }
}

impl std::ops::Sub for Timestamp {
    type Output = Duration;

    fn sub(self, rhs: Timestamp) -> Self::Output {
        Duration::from_millis((self.0 - rhs.0) as u64)
    }
}
