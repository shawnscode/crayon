use resource::errors::*;
use resource::{ResourceFrontend, Material, MaterialPtr};

const BUILTIN_SPRITE_PATH: &'static str = "_CRAYON_/material/sprite";

pub fn sprite(mut frontend: &mut ResourceFrontend) -> Result<MaterialPtr> {
    if let Some(rc) = frontend.get(BUILTIN_SPRITE_PATH) {
        return Ok(rc);
    }

    let shader = super::shader::sprite(&mut frontend)?;
    let mat = Material::new(shader);

    frontend.insert(BUILTIN_SPRITE_PATH, mat)
}