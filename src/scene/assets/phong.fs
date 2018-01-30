#version 100
#define MAX_POINT_LIGHTS 4

precision lowp float;

varying vec3 v_EyeFragPos;
varying vec3 v_EyeNormal;
varying vec4 v_Color;

uniform vec3 scn_DirLightViewDir;
uniform vec3 scn_DirLightColor;
uniform vec3 scn_PointLightViewPos[MAX_POINT_LIGHTS];
uniform vec3 scn_PointLightColor[MAX_POINT_LIGHTS];
uniform vec3 scn_PointLightAttenuation[MAX_POINT_LIGHTS];

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
    vec3 reflectDir = reflect(-scn_DirLightViewDir, normal);
    vec3 result = CalculateLight(normal, viewDir, scn_DirLightViewDir, reflectDir) * scn_DirLightColor;

    // point lights
    for(int i = 0; i < MAX_POINT_LIGHTS; i++)
    {
        vec3 lightDir2 = normalize(v_EyeFragPos - scn_PointLightViewPos[i]);
        vec3 reflectDir2 = reflect(-lightDir2, normal);
        float distance = length(scn_PointLightViewPos[i] - v_EyeFragPos);
        float attenuation =
            scn_PointLightAttenuation[i].x +
            scn_PointLightAttenuation[i].y * distance +
            scn_PointLightAttenuation[i].z * (distance * distance);

        vec3 power = CalculateLight(normal, viewDir, lightDir2, reflectDir2) * scn_PointLightColor[i];
        result += max(power * attenuation, vec3(0.0, 0.0, 0.0));
    }

    gl_FragColor = vec4(result, 1.0) * v_Color;
}