use std::collections::HashMap;

use graphics;
use resource::errors::*;
use resource::{ResourceFrontend, Shader, ShaderPtr};

const BUILTIN_SPRITE_PATH: &'static str = "_CRAYON_/shader/sprite";
const BUILTIN_PHONG_PATH: &'static str = "_CRAYON_/shader/phong";
const BUILTIN_COLOR_PATH: &'static str = "_CRAYON_/shader/color";

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

uniform sampler2D bi_MainTex;

void main() {
    gl_FragColor = v_Additive + v_Color * texture2D(bi_MainTex, v_Texcoord);
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
    uniforms.insert("bi_MainTex".to_owned(), UVT::Texture);

    let sprite = Shader::new(vs, fs, state, layout, uniforms);
    frontend.insert(BUILTIN_SPRITE_PATH, sprite)
}

pub fn phong(frontend: &mut ResourceFrontend) -> Result<ShaderPtr> {
    if let Some(rc) = frontend.get(BUILTIN_PHONG_PATH) {
        return Ok(rc);
    }

    let vs = "
#version 100
precision lowp float;

attribute vec3 Position;
attribute vec4 Color0;
attribute vec3 Normal;
attribute vec2 Texcoord0;

uniform mat4 bi_ViewModelMatrix;
uniform mat4 bi_ProjectionMatrix;
uniform mat4 bi_NormalMatrix;

varying vec3 v_EyeFragPos;
varying vec3 v_EyeNormal;
varying vec2 v_Texcoord;
varying vec4 v_Color;

void main() {
    gl_Position = bi_ProjectionMatrix * bi_ViewModelMatrix * vec4(Position, 1.0);
    v_EyeFragPos = vec3(bi_ViewModelMatrix * vec4(Position, 1.0));
    v_EyeNormal = vec3(bi_NormalMatrix * vec4(Normal, 1.0));
    v_Texcoord = Texcoord0;
    v_Color = Color0;
}
    "
            .to_owned();

    let fs = "
#version 100
precision lowp float;

varying vec3 v_EyeFragPos;
varying vec3 v_EyeNormal;
varying vec2 v_Texcoord;
varying vec4 v_Color;

#define MAX_POINT_LIGHTS 4

uniform vec3 bi_DirLightEyeDir;
uniform vec3 bi_DirLightColor;

uniform vec3 bi_PointLightEyePos[MAX_POINT_LIGHTS];
uniform vec3 bi_PointLightColor[MAX_POINT_LIGHTS];
uniform vec3 bi_PointLightAttenuation[MAX_POINT_LIGHTS];

// Phong materials
uniform vec3 u_Ambient;
uniform vec3 u_Diffuse;
uniform vec3 u_Specular;
uniform float u_Shininess;

vec3 CalculateLight(vec3 normal, vec3 viewDir, vec3 lightDir, vec3 reflectDir)
{
    vec3 ambient = u_Ambient;
    vec3 diffuse = max(dot(normal, -lightDir), 0.0) * u_Diffuse;
    vec3 specular = pow(max(dot(viewDir, reflectDir), 0.0), u_Shininess) * u_Specular;
    return (0.2 * ambient + 0.5 * diffuse + specular);
}

void main()
{
    vec3 normal = normalize(v_EyeNormal);
    vec3 viewDir = normalize(v_EyeFragPos);

    // directional light
    vec3 reflectDir = reflect(-bi_DirLightEyeDir, normal);
    vec3 result = CalculateLight(normal, viewDir, bi_DirLightEyeDir, reflectDir) * bi_DirLightColor;

    // point lights
    for(int i = 0; i < MAX_POINT_LIGHTS; i++)
    {
        vec3 lightDir2 = normalize(v_EyeFragPos - bi_PointLightEyePos[i]);
        vec3 reflectDir2 = reflect(-lightDir2, normal);

        float distance = length(bi_PointLightEyePos[i] - v_EyeFragPos);
        float attenuation = 1.0 / (
            bi_PointLightAttenuation[i].x +
            bi_PointLightAttenuation[i].y * distance +
            bi_PointLightAttenuation[i].z * (distance * distance));

        result += CalculateLight(normal, viewDir, lightDir2, reflectDir2) * bi_PointLightColor[i] * attenuation;
    }

    gl_FragColor = vec4(result, 1.0) * v_Color;
}
    "
            .to_owned();

    let layout = graphics::AttributeLayoutBuilder::new()
        .with(graphics::VertexAttribute::Position, 3)
        .with(graphics::VertexAttribute::Normal, 3)
        .with(graphics::VertexAttribute::Texcoord0, 2)
        .with(graphics::VertexAttribute::Color0, 4)
        .finish();

    let mut state = graphics::RenderState::default();
    {
        use graphics::Comparison;
        state.depth_test = Comparison::Less;
        state.depth_write = true;

        state.cull_face = graphics::CullFace::Back;
        state.front_face_order = graphics::FrontFaceOrder::CounterClockwise;
    }

    use graphics::UniformVariableType as UVT;
    let mut uniforms = HashMap::new();
    uniforms.insert("bi_ViewModelMatrix".to_owned(), UVT::Matrix4f);
    uniforms.insert("bi_ProjectionMatrix".to_owned(), UVT::Matrix4f);
    uniforms.insert("bi_NormalMatrix".to_owned(), UVT::Matrix4f);

    uniforms.insert("bi_DirLightEyeDir".to_owned(), UVT::Vector3f);
    uniforms.insert("bi_DirLightColor".to_owned(), UVT::Vector3f);

    for i in 0..3 {
        uniforms.insert(format!("bi_PointLightEyePos[{:?}]", i), UVT::Vector3f);
        uniforms.insert(format!("bi_PointLightColor[{:?}]", i), UVT::Vector3f);
        uniforms.insert(format!("bi_PointLightAttenuation[{:?}]", i), UVT::Vector3f);
    }

    uniforms.insert("u_Ambient".to_owned(), UVT::Vector3f);
    uniforms.insert("u_Diffuse".to_owned(), UVT::Vector3f);
    uniforms.insert("u_Specular".to_owned(), UVT::Vector3f);
    uniforms.insert("u_Shininess".to_owned(), UVT::F32);

    let sprite = Shader::new(vs, fs, state, layout, uniforms);
    frontend.insert(BUILTIN_PHONG_PATH, sprite)
}

pub fn color(frontend: &mut ResourceFrontend) -> Result<ShaderPtr> {
    if let Some(rc) = frontend.get(BUILTIN_COLOR_PATH) {
        return Ok(rc);
    }

    let vs = "
#version 100
precision lowp float;

attribute vec3 Position;

uniform mat4 bi_ViewModelMatrix;
uniform mat4 bi_ProjectionMatrix;

void main() {
    gl_Position = bi_ProjectionMatrix * bi_ViewModelMatrix * vec4(Position, 1.0);
}
    "
            .to_owned();

    let fs = "
#version 100
precision lowp float;

uniform vec3 u_Color;

void main() {
    gl_FragColor = vec4(u_Color, 1.0);
}
    "
            .to_owned();

    let layout = graphics::AttributeLayoutBuilder::new()
        .with(graphics::VertexAttribute::Position, 3)
        .finish();

    let mut state = graphics::RenderState::default();
    {
        use graphics::Comparison;
        state.depth_test = Comparison::Less;
        state.depth_write = true;
    }

    use graphics::UniformVariableType as UVT;
    let mut uniforms = HashMap::new();
    uniforms.insert("bi_ViewModelMatrix".to_owned(), UVT::Matrix4f);
    uniforms.insert("bi_ProjectionMatrix".to_owned(), UVT::Matrix4f);

    uniforms.insert("u_Color".to_owned(), UVT::Vector3f);

    let sprite = Shader::new(vs, fs, state, layout, uniforms);
    frontend.insert(BUILTIN_COLOR_PATH, sprite)
}
