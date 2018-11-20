use crayon::errors::*;

use crayon::video;
use crayon::video::assets::texture::*;

pub fn white() -> Result<TextureHandle> {
    let mut params = TextureParams::default();
    params.dimensions = (2, 2).into();

    let bytes = vec![255; 16];
    let data = TextureData {
        bytes: vec![bytes.into_boxed_slice()],
    };

    let texture = video::create_texture(params, data)?;
    Ok(texture)
}
