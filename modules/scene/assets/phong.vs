#version 100
precision lowp float;

attribute vec3 Position;
attribute vec3 Normal;
attribute vec4 Color0;

uniform mat4 scn_ModelViewMatrix;
uniform mat4 scn_MVPMatrix;
uniform mat4 scn_ViewNormalMatrix;
uniform mat4 scn_ShadowCasterSpaceMatrix;

varying vec3 v_ShadowFragPos;
varying vec3 v_EyeFragPos;
varying vec3 v_EyeNormal;
varying vec4 v_Color;

void main() {
    gl_Position = scn_MVPMatrix * vec4(Position, 1.0);
    
    vec4 shadowPos = scn_ShadowCasterSpaceMatrix * vec4(Position, 1.0);
    v_ShadowFragPos = shadowPos.xyz / shadowPos.w;

    vec4 eyePos = scn_ModelViewMatrix * vec4(Position, 1.0);
    v_EyeFragPos = eyePos.xyz / eyePos.w;

    v_EyeNormal = vec3(scn_ViewNormalMatrix * vec4(Normal, 0.0));
    v_Color = Color0;
}