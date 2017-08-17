#version 330 core
in vec2 UV;
out vec3 color;
uniform sampler2D renderedTexture;
uniform float time;
void main() {
    vec2 offset = 0.025*vec2(sin(time+1024.0*UV.x), cos(time+768.0*UV.y));
    color = texture( renderedTexture, UV + offset ).xyz;
}