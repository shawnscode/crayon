#version 100
precision lowp float;

varying vec2 v_Texcoord;
varying vec4 v_Color;
varying vec4 v_AdditiveColor;

uniform sampler2D u_MainTex;

void main() {
    gl_FragColor = texture2D( u_MainTex, v_Texcoord ) * v_Color + v_AdditiveColor;
}