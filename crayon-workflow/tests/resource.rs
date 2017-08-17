extern crate crayon;
extern crate crayon_workflow;
extern crate image;

use crayon_workflow::resource;
use crayon_workflow::Manifest;

use std::path::Path;

#[test]
fn database() {
    use resource::ResourceDatabase;

    ///
    let manifest = Manifest::find("tests/workspace").unwrap().setup().unwrap();
    let mut database = ResourceDatabase::new(manifest).unwrap();
    database.refresh().unwrap();
    database.save().unwrap();

    ///
    {
        let path = Path::new("tests/workspace/resources/texture.png.meta");
        assert!(path.exists());

        let path = Path::new("tests/workspace/resources/invalid_texture.png.meta");
        assert!(!path.exists());

        assert!(database.len() == 1);
    }

    /// Make sure processed resources could be read at runtime.
    database
        .build("0.0.1",
               crayon_workflow::platform::BuildTarget::MacOS,
               "tests/build")
        .unwrap();

    assert!(database
                .uuid("tests/workspace/resources/invalid_texture.png")
                .is_none());

    let mut resource_system = crayon::resource::ResourceSystem::new().unwrap();

    {
        resource_system
            .load_manifest("tests/build/manifest")
            .unwrap();

        resource_system.load("texture.png").unwrap();
        resource_system.load_texture("texture.png").unwrap();

        assert!(resource_system.load_texture("invalid_texture.png").is_err());
    }

    {
        assert!(database
                    .uuid("tests/workspace/resources/invalid_resource.png")
                    .is_none());

        let uuid = database
            .uuid("tests/workspace/resources/texture.png")
            .unwrap();

        resource_system.load_with_uuid(uuid).unwrap();
        resource_system.load_texture_with_uuid(uuid).unwrap();
    }
}