extern crate crayon;

fn main() {
    crayon::Application::setup("resources/setup.json")
        .unwrap()
        .perform(|mut app| {
                     println!("Hello, Crayon!");
                 })
        .run(move |mut app| { return true; });
}
