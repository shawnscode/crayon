extern crate crayon_workflow;
extern crate image;

use crayon_workflow::resource;
use crayon_workflow::Manifest;

use std::path::Path;

#[test]
fn database() {
    use resource::ResourceDatabase;

    let manifest = Manifest::find("tests/workspace").unwrap().setup().unwrap();
    let mut database = ResourceDatabase::new(manifest).unwrap();
    database.refresh().unwrap();
    database.save().unwrap();

    database
        .build("0.0.1",
               crayon_workflow::platform::BuildTarget::MacOS,
               "tests/build")
        .unwrap();

    {
        let path = Path::new("tests/workspace/resources/texture.png.meta");
        assert!(path.exists());

        let path = Path::new("tests/workspace/resources/invalid_texture.png.meta");
        assert!(!path.exists());

        assert!(database.len() == 1);
    }
}