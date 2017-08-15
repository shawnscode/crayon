extern crate crayon_workflow;
extern crate image;

use crayon_workflow::resource;
use crayon_workflow::Manifest;

#[test]
fn database() {
    use resource::ResourceDatabase;

    let manifest = Manifest::find("tests/workspace").unwrap().setup().unwrap();
    let mut database = ResourceDatabase::new(manifest).unwrap();
    database.refresh().unwrap();
    database.save().unwrap();

    {
        let path = Path::new("tests/resources/texture.png.meta");
        assert!(path.exists());

        let path = Path::new("tests/resources/invalid_texture.png.meta");
        assert!(!path.exists());

        assert!(database.len() == 1);
    }
}