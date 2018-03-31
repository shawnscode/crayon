#version 100
#define MAX_DIR_LIGHTS 1

precision lowp float;

attribute vec3 Position;
attribute vec3 Normal;
attribute vec4 Color0;

uniform mat4 scn_ModelViewMatrix;
uniform mat4 scn_MVPMatrix;
uniform mat4 scn_ViewNormalMatrix;
uniform mat4 scn_DirLitShadowSpaceMatrix[MAX_DIR_LIGHTS];

varying vec3 v_DirLitShadowPos[MAX_DIR_LIGHTS];
varying vec3 v_EyeFragPos;
varying vec3 v_EyeNormal;
varying vec4 v_Color;

void main() {
    gl_Position = scn_MVPMatrix * vec4(Position, 1.0);
    
    for( int i = 0; i < MAX_DIR_LIGHTS; i ++ )
    {
        vec4 shadowPos = scn_DirLitShadowSpaceMatrix[i] * vec4(Position, 1.0);
        v_DirLitShadowPos[i] = shadowPos.xyz / shadowPos.w;
    }

    vec4 eyePos = scn_ModelViewMatrix * vec4(Position, 1.0);
    v_EyeFragPos = eyePos.xyz / eyePos.w;

    v_EyeNormal = vec3(scn_ViewNormalMatrix * vec4(Normal, 0.0));
    v_Color = Color0;
}