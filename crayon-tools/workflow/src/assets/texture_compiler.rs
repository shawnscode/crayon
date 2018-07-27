use std::io::{Read, Write};

use crayon::res::byteorder::ByteOrderWrite;
use crayon::video::assets::{texture, texture_loader};
use image::{self, GenericImage};

use super::ResourceCompiler;
use errors::*;

pub struct TextureCompiler {}

impl ResourceCompiler for TextureCompiler {
    fn compile(&self, i: &mut dyn Read, o: &mut dyn Write) -> Result<()> {
        let io_err = |err| Error::Compile(format!("[TextureCompiler] {}", err));
        let img_err = |err| Error::Compile(format!("[TextureCompiler] {}", err));

        let mut buf = Vec::new();
        i.read_to_end(&mut buf).map_err(io_err)?;

        let img = image::load_from_memory(&buf).map_err(img_err)?;

        // MAGIC: [u8; 8]
        o.write_all(&texture_loader::MAGIC).map_err(io_err)?;

        // PARAMS
        let mut params = texture::TextureParams::default();
        params.format = texture::TextureFormat::U8U8U8U8;
        params.dimensions = img.dimensions().into();

        unsafe {
            let mipmap = if params.mipmap { 1 } else { 0 };

            o.write_u8(::std::mem::transmute_copy(&params.wrap))
                .map_err(io_err)?;
            o.write_u8(::std::mem::transmute_copy(&params.filter))
                .map_err(io_err)?;
            o.write_u8(::std::mem::transmute_copy(&mipmap))
                .map_err(io_err)?;
            o.write_u8(::std::mem::transmute_copy(&params.format))
                .map_err(io_err)?;
            o.write_u32(params.dimensions.x).map_err(io_err)?;
            o.write_u32(params.dimensions.y).map_err(io_err)?;
        }

        // BYTES
        for v in img.pixels().map(|(_, _, p)| p) {
            o.write_all(&v.data)?;
        }

        Ok(())
    }
}
