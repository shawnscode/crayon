#version 150
in vec4 v_Color;
in vec2 v_Texcoord;

uniform sampler2D u_MainTex;
out vec4 color;

void main() {
    color = v_Color * texture(u_MainTex, v_Texcoord);
}