extern crate crayon_workflow;

use std::path::Path;

use crayon_workflow::resource::Resource;

#[test]
fn load() {
    let manifest = crayon_workflow::manifest::Manifest::find("tests/resources")
        .unwrap()
        .setup()
        .unwrap();

    assert_eq!(manifest.dir(), Path::new("tests/resources"));
    assert_eq!(manifest.resources.len(), 1);

    assert_eq!(manifest.types.get("png").unwrap(), &Resource::Texture);
    assert_eq!(manifest.types.get("tga").unwrap(), &Resource::Texture);
    assert_eq!(manifest.types.get("bmp").unwrap(), &Resource::Texture);
    assert_eq!(manifest.types.get("psd"), None);
    assert_eq!(manifest.types.get("bytes").unwrap(), &Resource::Bytes);
    assert_eq!(manifest.types.get("lua").unwrap(), &Resource::Bytes);
}