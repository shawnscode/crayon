//! Use shader asset to provide the custom graphic pipeline.
//!
//! # Shader
//!
//! ## Built-In Variables
//!
//! We provides a handful of built-in global variables for your shaders: things like
//! current object’s transformation matrices, light parameters, current time and so
//! on.
//!
//! 1. _ModelMatrix: Current model matrix.
//! 1. _ViewMatrix: Current view matrix.
//! 1. _ProjectionMatrix: Current projection matrix.
//! 1. _ViewModelMatrix: Current view * model matrix.
//! 1. _PVMMatrix: Current projection * view * model matrix.
//! 1. _NormalMatrix: The matrix that transforms normal into view space.
//! 1. _Time: Time since game started, use to animate things inside the shaders.
//! 1. _SinTime: Sine of time.
//! 1. _CosTime: Cosine of time.
//! 1. _FrameTime: Delta time since last frame.

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
                               video: &mut graphics::GraphicsSystem)
                               -> graphics::errors::Result<()> {
        if self.pso.is_none() {
            let v = video
                .create_pipeline(&self.vs, &self.fs, &self.render_state, &self.layout)?;
            self.pso = Some(v);
        }

        Ok(())
    }

    pub fn video_object(&self) -> Option<graphics::PipelineStateHandle> {
        self.pso.as_ref().map(|v| v.handle)
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

impl super::super::Resource for Shader {
    fn size(&self) -> usize {
        0
    }
}