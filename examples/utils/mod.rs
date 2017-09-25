use crayon_workflow::prelude::*;

pub fn compile() {
    // Get the example manifest `workspace.toml`.
    let workspace = Workspace::load_from("examples/workspace.toml").unwrap();

    // Build resources into `compiled-resources.'
    let os = BuildTarget::MacOS;

    workspace
        .reimport("examples/resources/atlas.json", ResourceType::Atlas)
        .unwrap();

    workspace.save().unwrap();
    workspace.build(os, "examples/compiled-resources").unwrap();
}