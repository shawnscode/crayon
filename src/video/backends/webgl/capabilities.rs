use web_sys::WebGl2RenderingContext as WebGL;

use crate::video::assets::texture::TextureFormat;

/// Represents the capabilities of the context.
///
/// Contrary to the state, these values never change.
#[derive(Debug)]
pub struct Capabilities {
    /// The list of OpenGL extensions support by this implementation.
    pub extensions: Extensions,
}

impl Capabilities {
    pub unsafe fn new(ctx: &WebGL) -> Result<Capabilities, failure::Error> {
        Ok(Capabilities {
            extensions: Extensions::parse(ctx)?,
        })
    }

    pub fn support_texture_format(&self, format: TextureFormat) -> bool {
        match format {
            TextureFormat::Etc2RGB4BPP | TextureFormat::Etc2RGBA8BPP => {
                self.extensions.webgl_compressed_texture_etc
            }
            TextureFormat::PvrtcRGB2BPP
            | TextureFormat::PvrtcRGB4BPP
            | TextureFormat::PvrtcRGBA2BPP
            | TextureFormat::PvrtcRGBA4BPP => self.extensions.webgl_compressed_texture_pvrtc,
            TextureFormat::S3tcDxt1RGB4BPP | TextureFormat::S3tcDxt5RGBA8BPP => {
                self.extensions.webgl_compressed_texture_s3tc
            }
            _ => true,
        }
    }
}

macro_rules! extensions {
    ($($string:expr => $field:ident,)+) => {
/// Contains data about the list of extensions.
        #[derive(Debug, Clone, Copy)]
        pub struct Extensions {
            $(
                pub $field: bool,
            )+
        }

/// Returns the list of extensions supported by the backend.
///
/// The version must match the one of the backend.
///
/// *Safety*: the OpenGL context corresponding to `gl` must be current in the thread.
///
/// ## Panic
///
/// Can panic if the version number doesn't match the backend, leading to unloaded functions
/// being called.
///
        impl Extensions {
            pub unsafe fn parse(ctx: &WebGL) -> Result<Extensions, failure::Error> {
                Ok(Extensions {
                    $(
                        $field: ctx.get_extension($string).unwrap().is_some(),
                    )+
                })
            }
        }
    }
}

extensions! {
    "WEBGL_compressed_texture_s3tc" => webgl_compressed_texture_s3tc,
    "WEBGL_compressed_texture_pvrtc" => webgl_compressed_texture_pvrtc,
    "WEBGL_compressed_texture_etc" => webgl_compressed_texture_etc,
}
