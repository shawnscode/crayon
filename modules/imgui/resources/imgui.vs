#version 100
precision lowp float;

uniform mat4 matrix;

attribute vec2 Position;
attribute vec2 Texcoord0;
attribute vec4 Color0;

varying vec4 v_Color0;
varying vec2 v_Texcoord0;

void main()
{
    v_Texcoord0 = Texcoord0;
    v_Color0 = Color0;
    gl_Position = matrix * vec4(Position.xy, 0.0, 1.0);
}
