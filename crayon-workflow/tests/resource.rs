extern crate crayon_workflow;

use crayon_workflow::resource;

#[test]
fn texture() {
    let metadata = resource::Metadata::new(resource::ResourceMetadata::Texture(resource::TextureMetadata::new()));
    metadata.serialize("tests/resources/v.png.meta").unwrap();
}