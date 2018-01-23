#version 100
#define MAX_POINT_LIGHTS 4

precision lowp float;

varying vec3 v_EyeFragPos;
varying vec3 v_EyeNormal;
varying vec4 v_Color;

uniform vec3 u_DirLightEyeDir;
uniform vec3 u_DirLightColor;
uniform vec3 u_PointLightEyePos[MAX_POINT_LIGHTS];
uniform vec3 u_PointLightColor[MAX_POINT_LIGHTS];
uniform vec3 u_PointLightAttenuation[MAX_POINT_LIGHTS];

// Phong materials
uniform vec3 u_Ambient;
uniform vec3 u_Diffuse;
uniform vec3 u_Specular;
uniform float u_Shininess;

vec3 CalculateLight(vec3 normal, vec3 viewDir, vec3 lightDir, vec3 reflectDir)
{
    vec3 ambient = u_Ambient;
    vec3 diffuse = max(dot(normal, -lightDir), 0.0) * u_Diffuse;
    vec3 specular = pow(max(dot(viewDir, reflectDir), 0.0), u_Shininess) * u_Specular;
    return (0.2 * ambient + 0.5 * diffuse + specular);
}

void main()
{
    vec3 normal = normalize(v_EyeNormal);
    vec3 viewDir = normalize(v_EyeFragPos);

    // directional light
    vec3 reflectDir = reflect(-u_DirLightEyeDir, normal);
    vec3 result = CalculateLight(normal, viewDir, u_DirLightEyeDir, reflectDir) * u_DirLightColor;

    // point lights
    for(int i = 0; i < MAX_POINT_LIGHTS; i++)
    {
        vec3 lightDir2 = normalize(v_EyeFragPos - u_PointLightEyePos[i]);
        vec3 reflectDir2 = reflect(-lightDir2, normal);
        float distance = length(u_PointLightEyePos[i] - v_EyeFragPos);
        float attenuation =
            u_PointLightAttenuation[i].x +
            u_PointLightAttenuation[i].y * distance +
            u_PointLightAttenuation[i].z * (distance * distance);

        vec3 power = CalculateLight(normal, viewDir, lightDir2, reflectDir2) * u_PointLightColor[i];
        result += max(power * attenuation, vec3(0.0, 0.0, 0.0));
    }

    gl_FragColor = vec4(result, 1.0) * v_Color;
}