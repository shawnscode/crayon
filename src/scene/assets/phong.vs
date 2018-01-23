#version 100
precision lowp float;

attribute vec3 Position;
attribute vec3 Normal;
attribute vec4 Color0;

uniform mat4 u_ModelViewMatrix;
uniform mat4 u_MVPMatrix;
uniform mat4 u_NormalMatrix;

varying vec3 v_EyeFragPos;
varying vec3 v_EyeNormal;
varying vec4 v_Color;

void main() {
    gl_Position = u_MVPMatrix * vec4(Position, 1.0);

    v_EyeFragPos = vec3(u_ModelViewMatrix * vec4(Position, 1.0));
    v_EyeNormal = vec3(u_NormalMatrix * vec4(Normal, 1.0));
    v_Color = Color0;
}