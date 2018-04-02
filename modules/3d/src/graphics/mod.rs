mod data;
mod shadow;
pub mod renderer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DrawOrder {
    Shadow = 0,
    Camera,
    Max,
}

#[derive(Debug, Clone, Copy)]
pub struct DrawSetup {
    pub max_dir_lits: usize,
    pub max_point_lits: usize,
    pub max_shadow_casters: usize,
    pub max_shadow_distance: f32,
    pub max_shadow_resolution: (u16, u16),
}

impl Default for DrawSetup {
    fn default() -> Self {
        DrawSetup {
            max_dir_lits: 1,
            max_point_lits: 4,
            max_shadow_casters: 1,
            max_shadow_distance: 100.0,
            max_shadow_resolution: (512, 512),
        }
    }
}

pub mod prelude {
    pub use graphics::{DrawOrder, DrawSetup};
    pub use graphics::renderer::Renderer;
}
