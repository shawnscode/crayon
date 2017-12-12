#version 100
precision lowp float;

uniform sampler2D texture;

varying vec4 v_Color0;
varying vec2 v_Texcoord0;

void main() {
    gl_FragColor = v_Color0 * texture2D(texture, v_Texcoord0.xy);
}