#version 150
in vec2 Position;
in vec4 Color0;

uniform mat4 u_View;
uniform mat4 u_Projection;

out vec4 v_Color;

void main() {
    gl_Position = u_Projection * u_View * vec4(Position, 0.0, 1.0);
    v_Color = Color0;
}