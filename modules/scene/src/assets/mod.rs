pub mod factory;
pub mod pipeline;
pub mod material;

pub mod prelude {
    pub use assets::pipeline::{PipelineHandle, PipelineParams, PipelineSetup,
                               PipelineUniformVariable};
    pub use assets::material::MaterialHandle;
    pub use assets::factory;
}
