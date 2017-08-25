use graphics;

#[derive(Debug)]
pub struct Shader {
    vs: String,
    fs: String,
    layout: graphics::AttributeLayout,
    render_state: graphics::RenderState,
    pso: Option<graphics::PipelineStateRef>,
}

impl Shader {
    pub fn new(vs: String,
               fs: String,
               render_state: &graphics::RenderState,
               layout: &graphics::AttributeLayout)
               -> Shader {
        Shader {
            vs: vs,
            fs: fs,
            layout: *layout,
            render_state: *render_state,
            pso: None,
        }
    }

    pub fn update_video_object(&mut self,
                               video: &mut graphics::Graphics)
                               -> graphics::errors::Result<()> {
        if self.pso.is_none() {
            let v = video
                .create_pipeline(&self.vs, &self.fs, &self.render_state, &self.layout)?;
            self.pso = Some(v);
        }

        Ok(())
    }

    pub fn layout(&self) -> &graphics::AttributeLayout {
        &self.layout
    }

    pub fn render_state(&self) -> &graphics::RenderState {
        &self.render_state
    }
}

impl super::Resource for Shader {
    fn size(&self) -> usize {
        0
    }
}