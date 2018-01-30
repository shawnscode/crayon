#version 330 core

layout(location = 0) in vec4 Position;
layout(location = 1) in vec4 Normal;
layout(location = 2) in vec2 Texcoord0;

uniform mat4 scn_MVPMatrix;
uniform mat4 scn_ModelViewMatrix;
uniform mat4 scn_ViewNormalMatrix;

out vec3 v_Position;
out vec3 v_Normal;
out vec2 v_Texcoord0;

void main()
{
    vec4 pos = scn_ModelViewMatrix * Position;
    v_Position = vec3(pos.xyz) / pos.w;
    v_Normal = normalize(vec3(scn_ViewNormalMatrix * vec4(Normal.xyz, 0.0)));
    v_Texcoord0 = Texcoord0;

    gl_Position = scn_MVPMatrix * Position;
}