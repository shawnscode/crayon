use std::collections::HashMap;

use graphics;
use resource::errors::*;
use resource::{ResourceFrontend, Shader, ShaderPtr};

const BUILTIN_SPRITE_PATH: &'static str = "_CRAYON_/shader/sprite";

pub fn sprite(frontend: &mut ResourceFrontend) -> Result<ShaderPtr> {
    if let Some(rc) = frontend.get(BUILTIN_SPRITE_PATH) {
        return Ok(rc);
    }

    let vs = "
#version 100
precision lowp float;

attribute vec3 Position;
attribute vec4 Color0;
attribute vec4 Color1;
attribute vec2 Texcoord0;

uniform mat4 bi_ViewMatrix;
uniform mat4 bi_ProjectionMatrix;

varying vec4 v_Color;
varying vec4 v_Additive;
varying vec2 v_Texcoord;

void main() {
    gl_Position = bi_ProjectionMatrix * bi_ViewMatrix * vec4(Position.xy, 0.0, 1.0);
    v_Color = Color0;
    v_Additive = Color1;
    v_Texcoord = Texcoord0;
}
    "
            .to_owned();

    let fs = "
#version 100
precision lowp float;

varying vec4 v_Color;
varying vec4 v_Additive;
varying vec2 v_Texcoord;

uniform sampler2D u_MainTex;

void main() {
    gl_FragColor = v_Additive + v_Color * texture2D(u_MainTex, v_Texcoord);
}
    "
            .to_owned();

    let layout = graphics::AttributeLayoutBuilder::new()
        .with(graphics::VertexAttribute::Position, 3)
        .with(graphics::VertexAttribute::Color0, 4)
        .with(graphics::VertexAttribute::Color1, 4)
        .with(graphics::VertexAttribute::Texcoord0, 2)
        .finish();

    let mut state = graphics::RenderState::default();
    {
        // Enable color blend with equation: src * srcAlpha + dest * (1-srcAlpha);
        use graphics::{Equation, BlendFactor, BlendValue};
        state.color_blend = Some((Equation::Add,
                                  BlendFactor::Value(BlendValue::SourceAlpha),
                                  BlendFactor::OneMinusValue(BlendValue::SourceAlpha)));
    }

    use graphics::UniformVariableType as UVT;
    let mut uniforms = HashMap::new();
    uniforms.insert("bi_ViewMatrix".to_owned(), UVT::Matrix4f);
    uniforms.insert("bi_ProjectionMatrix".to_owned(), UVT::Matrix4f);
    uniforms.insert("u_MainTex".to_owned(), UVT::Texture);

    let sprite = Shader::new(vs, fs, state, layout, uniforms);
    frontend.insert(BUILTIN_SPRITE_PATH, sprite)
}