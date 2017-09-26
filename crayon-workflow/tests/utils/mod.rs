use crayon_workflow::prelude::*;

pub fn load() -> Workspace {
    // Get the example manifest `workspace.toml`.
    let workspace = Workspace::find("tests/workspace").unwrap();

    workspace
        .load_with_desc("tests/workspace/resources/atlas.json",
                        TexturePackerAtlasDesc::default().into())
        .unwrap();

    workspace.save().unwrap();

    workspace
        .build(BuildTarget::MacOS, "tests/workspace/build")
        .unwrap();

    workspace
}