#version 150
in vec2 Position;
out vec2 f_Color;
void main() {
    gl_Position = vec4(Position, 0.0, 1.0);
    f_Color = Position;
}