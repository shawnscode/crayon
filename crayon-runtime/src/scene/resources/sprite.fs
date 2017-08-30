#version 100
precision lowp float;

varying vec4 v_Color;
varying vec4 v_Additive;
varying vec2 v_Texcoord;

uniform sampler2D u_MainTex;

void main() {
    gl_FragColor = v_Additive + v_Color * texture2D(u_MainTex, v_Texcoord);
}