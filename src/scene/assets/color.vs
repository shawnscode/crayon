#version 100
precision lowp float;
attribute vec3 Position;

uniform mat4 u_MVPMatrix;

void main() {
    gl_Position = u_MVPMatrix * vec4(Position, 1.0);
}