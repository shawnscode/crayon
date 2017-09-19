use crayon_workflow as workflow;

pub fn compile() {
    // Get the example manifest `workspace.toml`.
    let workspace = workflow::Workspace::load_from("examples/workspace.toml").unwrap();

    // Build resources into `compiled-resources.'
    let os = workflow::platform::BuildTarget::MacOS;

    workspace
        .reimport("examples/resources/atlas.json", workflow::Resource::Atlas)
        .unwrap();

    workspace.save().unwrap();
    workspace.build(os, "examples/compiled-resources").unwrap();
}