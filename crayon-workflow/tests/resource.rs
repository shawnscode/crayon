extern crate crayon_workflow;

use std::fs;
use std::path::Path;
use crayon_workflow::resource;

const REMOVE_FILES: bool = false;

#[test]
fn database() {
    // {
    //     let metadata = resource::Metadata::new(resource::ResourceMetadata::Texture(resource::TextureMetadata::new()));

    //     let path = Path::new("tests/resources/texture.png.meta");
    //     metadata.serialize(&path).unwrap();
    //     assert!(path.exists());
    // }

    // {
    //     let metadata = resource::Metadata::new(resource::ResourceMetadata::Atlas(resource::AtlasMetadata::new()));

    //     let path = Path::new("tests/resources/texture.atlas.meta");
    //     metadata.serialize(&path).unwrap();
    //     assert!(path.exists());
    // }

    // if REMOVE_FILES {
    //     fs::remove_file(&path).unwrap();
    // }
}