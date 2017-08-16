extern crate crayon_workflow;

use crayon_workflow::resource::Resource;

#[test]
fn load() {
    let manifest = crayon_workflow::manifest::Manifest::find("tests/workspace").unwrap();

    let wd = ::std::env::current_dir()
        .unwrap()
        .join("tests")
        .join("workspace");

    assert_eq!(manifest.dir(), &wd);
    assert_eq!(manifest.resources.len(), 1);

    assert_eq!(manifest.types.get("png").unwrap(), &Resource::Texture);
    assert_eq!(manifest.types.get("tga").unwrap(), &Resource::Texture);
    assert_eq!(manifest.types.get("bmp").unwrap(), &Resource::Texture);
    assert_eq!(manifest.types.get("psd"), None);
    assert_eq!(manifest.types.get("bytes").unwrap(), &Resource::Bytes);
    assert_eq!(manifest.types.get("lua").unwrap(), &Resource::Bytes);
}