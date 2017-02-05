extern crate lemon3d;

fn main() {
    lemon3d::Application::setup("examples/resources/configs/basic.json")
        .unwrap()
        .run(|_| {
            return true;
        })
        .perform(|_| println!("hello world."));
}
