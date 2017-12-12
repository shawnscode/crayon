/// A RGBA `Color`. Each color component is a floating point value
/// with a range from 0 to 1.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Color(pub f32, pub f32, pub f32, pub f32);

impl Into<u32> for Color {
    fn into(self) -> u32 {
        let color = self.clip();
        let mut encoded = ((color.0 / 1.0 * 255.0) as u32) << 24;
        encoded |= ((color.1 / 1.0 * 255.0) as u32) << 16;
        encoded |= ((color.2 / 1.0 * 255.0) as u32) << 8;
        encoded |= ((color.3 / 1.0 * 255.0) as u32) << 0;
        encoded
    }
}

impl From<u32> for Color {
    fn from(encoded: u32) -> Self {
        Color(((encoded >> 24) & 0xFF) as f32 / 255.0,
              ((encoded >> 16) & 0xFF) as f32 / 255.0,
              ((encoded >> 8) & 0xFF) as f32 / 255.0,
              ((encoded >> 0) & 0xFF) as f32 / 255.0)
    }
}

impl Into<[u8; 4]> for Color {
    fn into(self) -> [u8; 4] {
        [(self.0 / 1.0 * 255.0) as u8,
         (self.1 / 1.0 * 255.0) as u8,
         (self.2 / 1.0 * 255.0) as u8,
         (self.3 / 1.0 * 255.0) as u8]
    }
}

impl From<[u8; 4]> for Color {
    fn from(v: [u8; 4]) -> Self {
        Color(v[0] as f32 / 255.0,
              v[1] as f32 / 255.0,
              v[2] as f32 / 255.0,
              v[3] as f32 / 255.0)
    }
}

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        [self.0, self.1, self.2, self.3]
    }
}

impl Color {
    /// Creates `Color` from a u32 encoded `ARGB`.
    pub fn from_argb_u32(encoded: u32) -> Self {
        Color(((encoded >> 16) & 0xFF) as f32 / 255.0,
              ((encoded >> 8) & 0xFF) as f32 / 255.0,
              ((encoded >> 0) & 0xFF) as f32 / 255.0,
              ((encoded >> 24) & 0xFF) as f32 / 255.0)
    }

    /// Creates `Color` from a u32 encoded `ABGR`.
    pub fn from_abgr_u32(encoded: u32) -> Self {
        Color(((encoded >> 0) & 0xFF) as f32 / 255.0,
              ((encoded >> 8) & 0xFF) as f32 / 255.0,
              ((encoded >> 16) & 0xFF) as f32 / 255.0,
              ((encoded >> 24) & 0xFF) as f32 / 255.0)
    }

    /// Returns the `grayscale` representation of RGB values.
    pub fn grayscale(&self) -> f32 {
        self.0 * 0.299 + self.1 * 0.587 + self.2 * 0.114
    }

    /// Clip to [0.0, 1.0] range.
    pub fn clip(&self) -> Color {
        let mut color = *self;
        color.0 = clamp(self.0, 0.0, 1.0);
        color.1 = clamp(self.1, 0.0, 1.0);
        color.2 = clamp(self.2, 0.0, 1.0);
        color.3 = clamp(self.3, 0.0, 1.0);
        color
    }

    /// Truncate alpha channel.
    pub fn rgb(&self) -> [f32; 3] {
        [self.0, self.1, self.2]
    }
}

impl Color {
    pub fn white() -> Self {
        Color(1.0, 1.0, 1.0, 1.0)
    }

    pub fn gray() -> Self {
        Color(0.5, 0.5, 0.5, 1.0)
    }

    pub fn black() -> Self {
        Color(0.0, 0.0, 0.0, 1.0)
    }

    pub fn red() -> Self {
        Color(1.0, 0.0, 0.0, 1.0)
    }

    pub fn green() -> Self {
        Color(0.0, 1.0, 0.0, 1.0)
    }

    pub fn blue() -> Self {
        Color(0.0, 0.0, 1.0, 1.0)
    }

    pub fn cyan() -> Self {
        Color(0.0, 1.0, 1.0, 1.0)
    }

    pub fn magenta() -> Self {
        Color(1.0, 0.0, 1.0, 1.0)
    }

    pub fn yellow() -> Self {
        Color(1.0, 1.0, 0.0, 1.0)
    }

    pub fn transparent() -> Self {
        Color(0.0, 0.0, 0.0, 0.0)
    }
}

fn clamp(v: f32, min: f32, max: f32) -> f32 {
    let mut v = v;

    if v < min {
        v = min;
    }

    if v > max {
        v = max;
    }

    v
}