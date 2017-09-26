use crayon_workflow::prelude::*;

pub fn compile() {
    // Get the example manifest `workspace.toml`.
    let workspace = Workspace::load_from("examples/workspace.toml").unwrap();

    // Build resources into `compiled-resources.'
    let os = BuildTarget::MacOS;

    workspace
        .load_with_desc("examples/resources/atlas.json",
                        TexturePackerAtlasDesc::default().into())
        .unwrap();

    workspace.save().unwrap();
    workspace.build(os, "examples/compiled-resources").unwrap();
}