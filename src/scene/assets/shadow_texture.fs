#version 100
precision lowp float;

varying vec2 v_Texcoord;

uniform sampler2D u_ShadowTexture;

float LinearizeOrthoDepth(vec2 uv)
{
    return texture2D(u_ShadowTexture, uv).r;
}

void main() {
    float d = LinearizeOrthoDepth(v_Texcoord);
    gl_FragColor = vec4(vec3(d), 1.0);
}