#version 100
precision lowp float;

uniform sampler2D mainTexture;

varying vec4 v_Color0;
varying vec2 v_Texcoord0;

void main() {
    gl_FragColor = v_Color0 * vec4(1.0, 1.0, 1.0, texture2D(mainTexture, v_Texcoord0).r);
}