extern crate crayon;

fn main() {
    crayon::Application::setup("configs")
        .unwrap()
        .perform(|mut app| app.resources.load_manifest("resources/manifest").unwrap())
        .run(move |mut _app| true);
}
