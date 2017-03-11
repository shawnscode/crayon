#version 150
in vec2 f_Color;
out vec4 out_color;
void main() {
    out_color = vec4(f_Color, 0.0, 1.0);
}