use std::collections::HashMap;

use crayon::graphics;
use platform;

/// Texture format that used to build resource.
#[derive(Debug, Serialize, Deserialize)]
pub enum TextureFormat {
    RGB, // Color texture format, 8-bits per channel.
    RGBA, // Color with alpha texture format, 8-bits per channel.
    PVRTC4, // PowerVR (iOS) 4 bits/pixel compressed color texture format.
    PVRTC4A, // PowerVR (iOS) 4 bits/pixel compressed with alpha channel texture format.
}

/// Platform-independent settings of texture importer.
#[derive(Debug, Serialize, Deserialize)]
pub struct TexturePlatformMetadata {
    format: TextureFormat,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextureMetadata {
    pub mipmap: bool,
    pub address: graphics::TextureAddress,
    pub filter: graphics::TextureFilter,
    pub platform_settings: HashMap<platform::BuildTarget, TexturePlatformMetadata>,
}

impl TextureMetadata {
    pub fn new() -> TextureMetadata {
        let mut settings = HashMap::new();
        settings.insert(platform::BuildTarget::MacOS,
                        TexturePlatformMetadata { format: TextureFormat::PVRTC4A });

        TextureMetadata {
            address: graphics::TextureAddress::Clamp,
            filter: graphics::TextureFilter::Linear,
            mipmap: false,
            platform_settings: settings,
        }
    }
}