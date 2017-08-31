use std::collections::HashMap;
use graphics;

#[derive(Debug)]
pub struct Shader {
    vs: String,
    fs: String,
    render_state: graphics::RenderState,
    layout: graphics::AttributeLayout,
    uniforms: HashMap<String, graphics::UniformVariableType>,
    pso: Option<graphics::PipelineStateRef>,
}

impl Shader {
    pub fn new(vs: String,
               fs: String,
               render_state: graphics::RenderState,
               layout: graphics::AttributeLayout,
               uniforms: HashMap<String, graphics::UniformVariableType>)
               -> Shader {
        Shader {
            vs: vs,
            fs: fs,
            render_state: render_state,
            layout: layout,
            uniforms: uniforms,
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

    pub fn uniform_variable(&self, name: &str) -> Option<graphics::UniformVariableType> {
        self.uniforms.get(name).and_then(|v| Some(*v))
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