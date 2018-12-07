use gl;
use gl::types::*;
use std::cmp;
use std::ffi;
use std::mem;

use crate::errors::*;

/// Describes the OpenGL context profile.
#[derive(Debug, Copy, Clone)]
pub enum Profile {
    /// The context uses only future-compatible functions and definitions.
    Core,
    /// The context includes all immediate mode functions and definitions.
    Compatibility,
}

/// Describes a version.
///
/// A version can only be compared to another version if they belong to the same API.
/// For example, both `Version::GL(3, 0) >= Version::ES(3, 0)` and `Version::ES(3, 0) >=
/// Version::GL(3, 0)` return `false`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Version {
    /// Regular OpenGL.
    GL(u8, u8),
    /// OpenGL embedded system.
    ES(u8, u8),
}

impl PartialOrd for Version {
    #[inline]
    fn partial_cmp(&self, other: &Version) -> Option<cmp::Ordering> {
        let (es1, major1, minor1) = match *self {
            Version::GL(major, minor) => (false, major, minor),
            Version::ES(major, minor) => (true, major, minor),
        };

        let (es2, major2, minor2) = match *other {
            Version::GL(major, minor) => (false, major, minor),
            Version::ES(major, minor) => (true, major, minor),
        };

        if es1 != es2 {
            None
        } else {
            match major1.cmp(&major2) {
                cmp::Ordering::Equal => Some(minor1.cmp(&minor2)),
                v => Some(v),
            }
        }
    }
}

impl Version {
    /// Obtains the OpenGL version of the current context using the loaded functions.
    ///
    /// # Unsafe
    ///
    /// You must ensure that the functions belong to the current context, otherwise you will get
    /// an undefined behavior.
    pub unsafe fn parse() -> Result<Version> {
        let desc = gl::GetString(gl::VERSION);
        let desc = String::from_utf8(ffi::CStr::from_ptr(desc as *const _).to_bytes().to_vec())
            .map_err(|_| format_err!("[GL] String is unformaled."))?;

        let (es, desc) = if desc.starts_with("OpenGL ES ") {
            (true, &desc[10..])
        } else if desc.starts_with("OpenGL ES-") {
            (true, &desc[13..])
        } else {
            (false, &desc[..])
        };

        let desc = desc
            .split(' ')
            .next()
            .ok_or_else(|| format_err!("[GL] String is unformaled."))?;

        let mut iter = desc.split(move |c: char| c == '.');
        let major = iter.next().unwrap();
        let minor = iter.next().unwrap();

        let major = major.parse().expect("failed to parse GL major version");
        let minor = minor.parse().expect("failed to parse GL minor version");

        if es {
            Ok(Version::ES(major, minor))
        } else {
            Ok(Version::GL(major, minor))
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
            pub unsafe fn parse(version: Version) -> Result<Extensions> {
                let strings: Vec<String> = if version >= Version::GL(3, 0) || version >= Version::ES(3, 0) {
                    let mut num_extensions = 0;
                    gl::GetIntegerv(gl::NUM_EXTENSIONS, &mut num_extensions);
                    (0 .. num_extensions).map(|i| {
                        let ext = gl::GetStringi(gl::EXTENSIONS, i as gl::types::GLuint);
                        String::from_utf8(ffi::CStr::from_ptr(ext as *const _).to_bytes().to_vec()).unwrap()
                    }).collect()
                } else {
                    let list = gl::GetString(gl::EXTENSIONS);
                    assert!(!list.is_null());
                    let list = String::from_utf8(ffi::CStr::from_ptr(list as *const _).to_bytes().to_vec())
                                                .unwrap();
                    list.split(' ').map(|e| e.to_owned()).collect()
                };

                let mut extensions = Extensions {
                    $(
                        $field: false,
                    )+
                };

                for extension in strings {
                    match &extension[..] {
                        $(
                            $string => extensions.$field = true,
                        )+
                        _ => ()
                    }
                }

                Ok(extensions)
            }
        }
    }
}

extensions! {
    "GL_ARB_shader_objects" => gl_arb_shader_objects,
    "GL_ARB_vertex_shader" => gl_arb_vertex_shader,
    "GL_ARB_fragment_shader" => gl_arb_fragment_shader,
    "GL_ARB_vertex_buffer_object" => gl_arb_vertex_buffer_object,
    "GL_ARB_map_buffer_range" => gl_arb_map_buffer_range,
    "GL_ARB_uniform_buffer_object" => gl_arb_uniform_buffer_object,
    "GL_ARB_framebuffer_no_attachments" => gl_arb_framebuffer_no_attachments,
    "GL_ARB_framebuffer_object" => gl_arb_framebuffer_object,
    "GL_ARB_vertex_array_object" => gl_arb_vertex_array_object,
    "GL_APPLE_vertex_array_object" => gl_apple_vertex_array_object,
    "GL_EXT_framebuffer_object" => gl_ext_framebuffer_object,
    "GL_EXT_framebuffer_blit" => gl_ext_framebuffer_blit,
    "GL_NV_fbo_color_attachments" => gl_nv_fbo_color_attachments,
    "GL_OES_vertex_array_object" => gl_oes_vertex_array_object,
    "GL_IMG_texture_compression_pvrtc" => gl_img_texture_compression_pvrtc,
    "GL_EXT_texture_compression_s3tc" => gl_ext_texture_compression_s3tc,
    "GL_ARB_ES3_compatibility" => gl_arb_es3_compatibility,
    "GL_OES_compressed_ETC2_RGB8_texture" => gl_oes_compressed_etc2_rgb8_texture,
    "GL_OES_compressed_ETC2_RGBA8_texture" => gl_oes_compressed_etc2_rgba8_texture,
}

#[derive(Debug, Copy, Clone)]
pub enum TextureCompression {
    ETC2,
    PVRTC,
    S3TC,
}

/// Represents the capabilities of the context.
///
/// Contrary to the state, these values never change.
#[derive(Debug)]
pub struct Capabilities {
    /// Returns a version or release number. Vendor-specific information may follow the version
    /// number.
    pub version: Version,

    /// The company responsible for this GL implementation.
    pub vendor: String,

    /// The list of OpenGL extensions support by this implementation.
    pub extensions: Extensions,

    /// The name of the renderer. This name is typically specific to a particular
    /// configuration of a hardware platform.
    pub renderer: String,

    /// The OpenGL context profile if available.
    ///
    /// The context profile is available from OpenGL 3.2 onwards. `None` if not supported.
    pub profile: Option<Profile>,

    /// The context is in debug mode, which may have additional error and performance issue
    /// reporting functionality.
    pub debug: bool,

    /// The context is in "forward-compatible" mode, which means that no deprecated functionality
    /// will be supported.
    pub forward_compatible: bool,

    /// Maximum width and height of `glViewport`.
    pub max_viewport_dims: (u32, u32),

    /// Maximum number of textures that can be bound to a program.
    ///
    /// `glActiveTexture` must be between `GL_TEXTURE0` and `GL_TEXTURE0` + this value - 1.
    pub max_combined_texture_image_units: u8,

    /// Number of available buffer bind points for `GL_UNIFORM_BUFFER`.
    pub max_indexed_uniform_buffer: u32,

    /// Maximum number of color attachment bind points.
    pub max_color_attachments: u32,
}

impl Capabilities {
    pub unsafe fn parse() -> Result<Capabilities> {
        let version = Version::parse()?;
        let extensions = Extensions::parse(version)?;

        let (debug, forward_compatible) = if version >= Version::GL(3, 0) {
            let mut val = mem::uninitialized();
            gl::GetIntegerv(gl::CONTEXT_FLAGS, &mut val);
            let val = val as gl::types::GLenum;
            (
                (val & gl::CONTEXT_FLAG_DEBUG_BIT) != 0,
                (val & gl::CONTEXT_FLAG_FORWARD_COMPATIBLE_BIT) != 0,
            )
        } else {
            (false, false)
        };

        Ok(Capabilities {
            version,
            extensions,
            vendor: Capabilities::parse_str(gl::VENDOR)?,
            renderer: Capabilities::parse_str(gl::RENDERER)?,
            profile: Capabilities::parse_profile(version),
            debug,
            forward_compatible,
            max_viewport_dims: Capabilities::parse_viewport_dims(),
            max_combined_texture_image_units: Capabilities::parse_texture_image_units(),
            max_indexed_uniform_buffer: Capabilities::parse_uniform_buffers(version, &extensions),
            max_color_attachments: Capabilities::parse_color_attachments(version, &extensions),
        })
    }

    pub fn has_compression(&self, compression: TextureCompression) -> bool {
        match compression {
            TextureCompression::ETC2 => {
                self.version >= Version::ES(3, 0)
                    || self.extensions.gl_arb_es3_compatibility
                    || (self.extensions.gl_oes_compressed_etc2_rgb8_texture
                        && self.extensions.gl_oes_compressed_etc2_rgba8_texture)
            }
            TextureCompression::PVRTC => self.extensions.gl_img_texture_compression_pvrtc,
            TextureCompression::S3TC => self.extensions.gl_ext_texture_compression_s3tc,
        }
    }

    #[inline]
    unsafe fn parse_str(id: GLenum) -> Result<String> {
        let s = gl::GetString(gl::RENDERER);
        if s.is_null() {
            bail!("[GL] String of {} is null.", id);
        }

        String::from_utf8(ffi::CStr::from_ptr(s as *const _).to_bytes().to_vec())
            .map_err(|_| format_err!("[GL] String of {} is unformaled.", id))
    }

    #[inline]
    unsafe fn parse_viewport_dims() -> (u32, u32) {
        let mut val: [gl::types::GLint; 2] = [0, 0];
        gl::GetIntegerv(gl::MAX_VIEWPORT_DIMS, val.as_mut_ptr());
        (val[0] as u32, val[1] as u32)
    }

    #[inline]
    unsafe fn parse_profile(version: Version) -> Option<Profile> {
        if version >= Version::GL(3, 2) {
            let mut val = mem::uninitialized();
            gl::GetIntegerv(gl::CONTEXT_PROFILE_MASK, &mut val);
            let val = val as gl::types::GLenum;
            if (val & gl::CONTEXT_COMPATIBILITY_PROFILE_BIT) != 0 {
                Some(Profile::Compatibility)
            } else if (val & gl::CONTEXT_CORE_PROFILE_BIT) != 0 {
                Some(Profile::Core)
            } else {
                None
            }
        } else {
            None
        }
    }

    #[inline]
    unsafe fn parse_texture_image_units() -> u8 {
        let mut val = 2;
        gl::GetIntegerv(gl::MAX_COMBINED_TEXTURE_IMAGE_UNITS, &mut val);
        val as u8
    }

    #[inline]
    unsafe fn parse_uniform_buffers(version: Version, exts: &Extensions) -> u32 {
        if version >= Version::GL(3, 1) || exts.gl_arb_uniform_buffer_object {
            let mut val = mem::uninitialized();
            gl::GetIntegerv(gl::MAX_UNIFORM_BUFFER_BINDINGS, &mut val);
            val as u32
        } else {
            0
        }
    }

    #[inline]
    unsafe fn parse_color_attachments(version: Version, exts: &Extensions) -> u32 {
        if version >= Version::GL(3, 0)
            || version >= Version::ES(3, 0)
            || exts.gl_arb_framebuffer_object
            || exts.gl_ext_framebuffer_object
            || exts.gl_nv_fbo_color_attachments
        {
            let mut val = 4;
            gl::GetIntegerv(gl::MAX_COLOR_ATTACHMENTS, &mut val);
            val as u32
        } else if version >= Version::ES(2, 0) {
            1
        } else {
            0
        }
    }
}
