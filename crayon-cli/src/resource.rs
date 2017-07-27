use std::path::Path;
use std::fs;
use crayon_workflow;

use errors::*;

pub fn refresh(man: &crayon_workflow::Manifest) -> Result<()> {
    for path in &man.resources {
        refresh_recursive(&man, &path)?;
    }

    Ok(())
}

pub fn refresh_recursive(man: &crayon_workflow::Manifest, path: &Path) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let sub_path = entry?.path();
        if sub_path.is_dir() {
            refresh_recursive(&man, &sub_path)?;
        } else {
            let mut metadata_path = sub_path.to_owned();
            let exts = String::from(metadata_path
                                        .extension()
                                        .unwrap_or("".as_ref())
                                        .to_string_lossy()) + ".meta";

            metadata_path.set_extension(&exts);

            if !fs::metadata(&metadata_path).is_ok() {
                import(&man, &sub_path, &metadata_path)?;
            }
        }
    }

    Ok(())
}

pub fn import(man: &crayon_workflow::Manifest, path: &Path, metadata_path: &Path) -> Result<()> {
    if let Some(v) = path.extension()
           .and_then(|v| man.types.get(v.to_str().unwrap())) {
        match v {
            &crayon_workflow::Resource::Texture => {
                let metadata =
                    crayon_workflow::Metadata::new(crayon_workflow::ResourceMetadata::Texture(crayon_workflow::TextureMetadata::new()));
                metadata.serialize(&metadata_path)?;
            }
            _ => (),
        }
    }

    Ok(())
}