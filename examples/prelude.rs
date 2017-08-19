use crayon_workflow as workflow;

pub fn compile() {
    // Get the example manifest `Crayon.toml`.
    let manifest = workflow::Manifest::load_from("examples/resources.toml")
        .unwrap()
        .setup()
        .unwrap();

    // Build resources into `compiled-resources.'
    let mut database = workflow::resource::ResourceDatabase::new(manifest).unwrap();
    let os = workflow::platform::BuildTarget::MacOS;

    database
        .load_metadata_as("examples/resources/atlas.json",
                          workflow::resource::Resource::Atlas)
        .unwrap();

    database.refresh().unwrap();
    database.save().unwrap();
    database
        .build("0.0.1", os, "examples/compiled-resources")
        .unwrap();
}