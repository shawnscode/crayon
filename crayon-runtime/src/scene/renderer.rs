pub trait Renderable {
    fn visible(&self) -> bool;
    fn set_visible(&mut self, visible: bool);
}

// pub struct Renderer {}

// impl Renderer {
//     pub fn new(app)
// }