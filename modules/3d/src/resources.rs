use crayon::application::Context;
use crayon::resource::utils::prelude::*;
use crayon::video::prelude::*;

use assets::material::Material;
use assets::pipeline::PipelineParams;
use assets::prelude::*;
use errors::*;

pub struct Resources {
    video: VideoSystemGuard,
    pub(crate) materials: Registery<Material>,
    pub(crate) pipelines: Registery<PipelineParams>,
}

impl Resources {
    pub fn new(ctx: &Context) -> Self {
        Resources {
            video: VideoSystemGuard::new(ctx.video.clone()),
            pipelines: Registery::new(),
            materials: Registery::new(),
        }
    }

    /// Lookups pipeline object from location.
    pub fn lookup_pipeline(&self, location: Location) -> Option<PipelineHandle> {
        self.pipelines.lookup(location).map(|v| v.into())
    }

    /// Creates a new pipeline object that indicates the whole render pipeline of `Scene`.
    pub fn create_pipeline(&mut self, setup: PipelineSetup) -> Result<PipelineHandle> {
        if let Some(handle) = self.lookup_pipeline(setup.location()) {
            self.pipelines.inc_rc(handle);
            return Ok(handle.into());
        }

        let (location, setup, links) = setup.into();
        let params = setup.params.clone();
        let shader = self.video.create_shader(setup)?;

        Ok(self.pipelines
            .create(location, PipelineParams::new(shader, params, links))
            .into())
    }

    /// Deletes a pipelie object.
    pub fn delete_pipeline(&mut self, handle: PipelineHandle) {
        self.pipelines.dec_rc(handle);
    }

    /// Creates a new material instance from shader.
    pub fn create_material(&mut self, setup: MaterialSetup) -> Result<MaterialHandle> {
        if let Some(po) = self.pipelines.get(setup.pipeline) {
            let location = Location::unique("");
            let material = Material::new(setup.pipeline, setup.variables, po.shader_params.clone());
            Ok(self.materials.create(location, material).into())
        } else {
            Err(Error::PipelineHandleInvalid(setup.pipeline))
        }
    }

    /// Gets the reference to material.
    pub fn material(&self, h: MaterialHandle) -> Option<&Material> {
        self.materials.get(h)
    }

    /// Gets the mutable reference to material.
    pub fn material_mut(&mut self, h: MaterialHandle) -> Option<&mut Material> {
        self.materials.get_mut(h)
    }

    /// Deletes the material instance from `Scene`. Any meshes that associated with a
    /// invalid/deleted material handle will be drawed with a fallback material marked
    /// with purple color.
    #[inline]
    pub fn delete_material(&mut self, handle: MaterialHandle) {
        self.materials.dec_rc(handle);
    }
}
