extern crate crayon;
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

    assert_eq!(manifest.settings.engine.min_fps, 20);
    assert_eq!(manifest.settings.engine.max_fps, 60);
    assert_eq!(manifest.settings.engine.time_smooth_step, 10);

    assert_eq!(manifest.settings.window.title, "Hello, Crayon!");
    assert_eq!(manifest.settings.window.width, 640);
    assert_eq!(manifest.settings.window.height, 320);

    /// Make sure processed configs could be read at runtime.
    manifest.save_settings("tests/build/configs").unwrap();
    crayon::core::settings::Settings::load_from("tests/build/configs").unwrap();
}