use crayon::errors::*;

use crayon::application::prelude::Context;
use crayon::video::assets::texture::*;

pub struct WorldBuiltinTextures {
    pub white: TextureHandle,
}

impl WorldBuiltinTextures {
    pub fn new(ctx: &Context) -> Result<Self> {
        Ok(WorldBuiltinTextures { white: white(ctx)? })
    }
}

fn white(ctx: &Context) -> Result<TextureHandle> {
    let mut params = TextureParams::default();
    params.dimensions = (2, 2).into();

    let bytes = vec![255; 16];
    let data = TextureData {
        bytes: vec![bytes.into_boxed_slice()],
    };

    let texture = ctx.video.create_texture(params, data)?;
    Ok(texture)
}
