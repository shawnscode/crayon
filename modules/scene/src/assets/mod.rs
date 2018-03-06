pub mod factory;
pub mod pipeline;
pub mod material;

pub mod prelude {
    pub use assets::pipeline::{PipelineHandle, PipelineSetup, PipelineUniformVariable};
    pub use assets::material::{MaterialHandle, MaterialSetup};
    pub use assets::factory;
}
