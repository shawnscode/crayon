pub mod factory;
pub mod material;
pub mod pipeline;

pub mod prelude {
    pub use assets::factory;
    pub use assets::material::{Material, MaterialHandle, MaterialSetup};
    pub use assets::pipeline::{PipelineHandle, PipelineSetup, PipelineUniformVariable};
}
