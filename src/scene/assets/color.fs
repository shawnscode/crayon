#version 100
precision lowp float;

uniform vec4 u_Color;

void main() {
    gl_FragColor = u_Color;
}