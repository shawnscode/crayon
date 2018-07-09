pub mod capabilities;
pub mod types;
pub mod visitor;

mod glutin {
    use gl;

    use super::super::super::errors::*;
    use super::visitor::GLVisitor;
    use application::window::Window;

    impl GLVisitor {
        pub unsafe fn glutin(window: &Window) -> Result<Self> {
            gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
            Self::new()
        }
    }
}
