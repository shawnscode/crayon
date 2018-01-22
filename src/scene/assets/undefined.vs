#version 100
precision lowp float;

attribute vec4 Position;

uniform mat4 u_MVPMatrix;

void main()
{
    gl_Position = u_MVPMatrix * Position;
}