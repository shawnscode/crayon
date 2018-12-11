use cgmath::BaseFloat;

/// A RGBA `Color`. Each color component is a floating point value
/// with a range from 0 to 1.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Color<S> {
    pub r: S,
    pub g: S,
    pub b: S,
    pub a: S,
}

impl Into<u32> for Color<f32> {
    fn into(self) -> u32 {
        let color = self.clip();
        let mut encoded = ((color.r / 1.0 * 255.0) as u32) << 24;
        encoded |= ((color.g / 1.0 * 255.0) as u32) << 16;
        encoded |= ((color.b / 1.0 * 255.0) as u32) << 8;
        encoded |= (color.a / 1.0 * 255.0) as u32;
        encoded
    }
}

impl<S: BaseFloat> From<u32> for Color<S> {
    fn from(encoded: u32) -> Self {
        let max = S::from(255.0).unwrap();
        Color::new(
            S::from((encoded >> 24) & 0xFF).unwrap() / max,
            S::from((encoded >> 16) & 0xFF).unwrap() / max,
            S::from((encoded >> 8) & 0xFF).unwrap() / max,
            S::from(encoded & 0xFF).unwrap() / max,
        )
    }
}

impl Into<[u8; 4]> for Color<f32> {
    fn into(self) -> [u8; 4] {
        let v = self.clip();
        let max = 255.0;
        [
            (v.r * max) as u8,
            (v.g * max) as u8,
            (v.b * max) as u8,
            (v.a * max) as u8,
        ]
    }
}

impl<S: BaseFloat> From<[u8; 4]> for Color<S> {
    fn from(v: [u8; 4]) -> Self {
        let max = S::from(255.0).unwrap();
        Color::new(
            S::from(v[0]).unwrap() / max,
            S::from(v[1]).unwrap() / max,
            S::from(v[2]).unwrap() / max,
            S::from(v[3]).unwrap() / max,
        )
    }
}

impl<S: BaseFloat> Color<S> {
    pub fn new(r: S, g: S, b: S, a: S) -> Self {
        Color { r, g, b, a }
    }

    /// Creates `Color` from a u32 encoded `ARGB`.
    pub fn from_argb_u32(encoded: u32) -> Self {
        let max = S::from(255.0).unwrap();
        Color::new(
            S::from((encoded >> 16) & 0xFF).unwrap() / max,
            S::from((encoded >> 8) & 0xFF).unwrap() / max,
            S::from(encoded & 0xFF).unwrap() / max,
            S::from((encoded >> 24) & 0xFF).unwrap() / max,
        )
    }

    /// Creates `Color` from a u32 encoded `ABGR`.
    pub fn from_abgr_u32(encoded: u32) -> Self {
        let max = S::from(255.0).unwrap();
        Color::new(
            S::from(encoded & 0xFF).unwrap() / max,
            S::from((encoded >> 8) & 0xFF).unwrap() / max,
            S::from((encoded >> 16) & 0xFF).unwrap() / max,
            S::from((encoded >> 24) & 0xFF).unwrap() / max,
        )
    }

    /// Returns the `grayscale` representation of RGB values.
    pub fn grayscale(&self) -> S {
        let fr = S::from(0.299).unwrap();
        let fg = S::from(0.587).unwrap();
        let fb = S::from(0.114).unwrap();

        self.r * fr + self.g * fg + self.b * fb
    }

    /// Clip to [0.0, 1.0] range.
    pub fn clip(&self) -> Self {
        let mut color = *self;
        color.r = self.r.max(S::zero()).min(S::one());
        color.g = self.g.max(S::zero()).min(S::one());
        color.b = self.b.max(S::zero()).min(S::one());
        color.a = self.a.max(S::zero()).min(S::one());
        color
    }

    /// Truncate alpha channel.
    pub fn rgb(&self) -> [S; 3] {
        [self.r, self.g, self.b]
    }

    pub fn rgba(&self) -> [S; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl<S: BaseFloat> Color<S> {
    pub fn white() -> Self {
        Color::new(S::one(), S::one(), S::one(), S::one())
    }

    pub fn gray() -> Self {
        let half = S::from(0.5).unwrap();
        Color::new(half, half, half, S::one())
    }

    pub fn black() -> Self {
        Color::new(S::zero(), S::zero(), S::zero(), S::one())
    }

    pub fn red() -> Self {
        Color::new(S::one(), S::zero(), S::zero(), S::one())
    }

    pub fn green() -> Self {
        Color::new(S::zero(), S::one(), S::zero(), S::one())
    }

    pub fn blue() -> Self {
        Color::new(S::zero(), S::zero(), S::one(), S::one())
    }

    pub fn cyan() -> Self {
        Color::new(S::zero(), S::one(), S::one(), S::one())
    }

    pub fn magenta() -> Self {
        Color::new(S::one(), S::zero(), S::one(), S::one())
    }

    pub fn yellow() -> Self {
        Color::new(S::one(), S::one(), S::zero(), S::one())
    }

    pub fn transparent() -> Self {
        Color::new(S::zero(), S::zero(), S::zero(), S::zero())
    }
}
