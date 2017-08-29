extern crate crayon;
extern crate crayon_workflow;

use crayon_workflow::resource::Resource;
use crayon_workflow::workspace::manifest::Manifest;

#[test]
fn load() {
    let manifest = Manifest::load_from("tests/workspace/workspace.toml").unwrap();

    let wd = ::std::env::current_dir()
        .unwrap()
        .join("tests")
        .join("workspace");

    {
        assert_eq!(manifest.dir(), &wd);
    }

    {
        let workspace = manifest.workspace();

        assert_eq!(workspace.resource_folders.len(), 1);
        assert_eq!(workspace.resource_exts.get("png").unwrap(),
                   &Resource::Texture);
        assert_eq!(workspace.resource_exts.get("tga").unwrap(),
                   &Resource::Texture);
        assert_eq!(workspace.resource_exts.get("bmp").unwrap(),
                   &Resource::Texture);
        assert_eq!(workspace.resource_exts.get("psd"), None);
        assert_eq!(workspace.resource_exts.get("bytes").unwrap(),
                   &Resource::Bytes);
        assert_eq!(workspace.resource_exts.get("lua").unwrap(),
                   &Resource::Bytes);
    }

    {
        let runtime = manifest.runtime();
        assert_eq!(runtime.engine.min_fps, 20);
        assert_eq!(runtime.engine.max_fps, 60);
        assert_eq!(runtime.engine.time_smooth_step, 10);

        assert_eq!(runtime.window.title, "Hello, Crayon!");
        assert_eq!(runtime.window.width, 640);
        assert_eq!(runtime.window.height, 320);
    }
}