use resource::errors::*;
use resource::{ResourceFrontend, Material, MaterialPtr};

use std::sync::{Arc, RwLock};

pub fn sprite(mut frontend: &mut ResourceFrontend) -> Result<MaterialPtr> {
    let shader = super::shader::sprite(&mut frontend)?;
    let mat = Material::new(shader);

    Ok(Arc::new(RwLock::new(mat)))
}

pub fn phong(mut frontend: &mut ResourceFrontend) -> Result<MaterialPtr> {
    let shader = super::shader::phong(&mut frontend)?;
    let mat = Material::new(shader);

    Ok(Arc::new(RwLock::new(mat)))
}

pub fn color(mut frontend: &mut ResourceFrontend) -> Result<MaterialPtr> {
    let shader = super::shader::color(&mut frontend)?;
    let mat = Material::new(shader);

    Ok(Arc::new(RwLock::new(mat)))
}