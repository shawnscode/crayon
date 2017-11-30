#version 100
precision lowp float;

attribute vec2 Position;
attribute vec2 Texcoord0;
attribute vec4 Color0;

// uniform mat4 transform;

varying vec4 v_Color0;
varying vec2 v_Texcoord0;

void main()
{
    gl_Position = vec4(Position, 0.0, 1.0);
    v_Texcoord0 = Texcoord0;
    v_Color0 = Color0;
}
