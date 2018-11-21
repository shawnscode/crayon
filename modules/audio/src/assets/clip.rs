impl_handle!(AudioClipHandle);

#[derive(Debug, Clone)]
pub struct AudioClip {
    pub pcm: Vec<i16>,
    pub channels: u8,
    pub sample_rate: u32,
}
