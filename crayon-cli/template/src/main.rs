extern crate crayon;

fn main() {
    crayon::Application::new()
        .unwrap()
        .perform(|mut app| app.resources.load_manifest("resources/manifest").unwrap())
        .run(move |mut _app| true);
}
