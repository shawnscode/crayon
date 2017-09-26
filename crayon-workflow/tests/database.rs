extern crate crayon;
extern crate crayon_workflow;
extern crate image;

mod utils;

use std::path::Path;

#[test]
fn database() {
    let workspace = utils::load();

    {
        let path = Path::new("tests/workspace/resources/texture.png.meta");
        assert!(path.exists());

        let path = Path::new("tests/workspace/resources/invalid_texture.png.meta");
        assert!(path.exists());
    }

    let mut rs = crayon::resource::ResourceFrontend::new().unwrap();

    {
        rs.load_manifest("tests/workspace/build/manifest").unwrap();

        let _: crayon::resource::TexturePtr = rs.load("texture.png").unwrap();
        let _: crayon::resource::BytesPtr = rs.load("invalid_texture.png").unwrap();
        assert!(rs.load::<crayon::resource::Texture, &str>("invalid_texture.png")
                    .is_err());
    }

    {
        let uuid = workspace
            .database
            .uuid("tests/workspace/resources/texture.png")
            .unwrap();

        rs.load_with_uuid::<crayon::resource::Texture>(uuid)
            .unwrap();
    }

    {
        let atlas: crayon::resource::AtlasPtr = rs.load("atlas.json").unwrap();
        let uuid = atlas.read().unwrap().texture();
        rs.load_with_uuid::<crayon::resource::Texture>(uuid)
            .unwrap();
    }
}