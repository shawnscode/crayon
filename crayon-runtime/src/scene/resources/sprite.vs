#version 100
precision lowp float;

attribute vec3 Position;
attribute vec4 Color0;
attribute vec4 Color1;
attribute vec2 Texcoord0;

uniform mat4 u_View;
uniform mat4 u_Proj;

varying vec4 v_Color;
varying vec4 v_Additive;
varying vec2 v_Texcoord;

void main() {
    gl_Position = u_Proj * u_View * vec4(Position.xy, 0.0, 1.0);
    v_Color = Color0;
    v_Additive = Color1;
    v_Texcoord = Texcoord0;
}