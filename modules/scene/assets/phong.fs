#version 100
#define MAX_POINT_LIGHTS 4

precision lowp float;

varying vec3 v_ShadowFragPos;
varying vec3 v_EyeFragPos;
varying vec3 v_EyeNormal;
varying vec4 v_Color;

uniform sampler2D scn_ShadowTexture;
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

vec3 CalculateLight(vec3 normal, vec3 viewDir, vec3 lightDir, vec3 reflectDir, float shadow)
{
    vec3 ambient = u_Ambient;
    vec3 diffuse = max(dot(normal, -lightDir), 0.0) * u_Diffuse;
    vec3 specular = pow(max(dot(viewDir, reflectDir), 0.0), u_Shininess) * u_Specular;
    return 0.2 * ambient + (1.0 - shadow) * (0.5 * diffuse + specular);
}

float CalculateShadow(vec3 shadowPos, float bias)
{
    // transform to [0,1] range.
    shadowPos = shadowPos * 0.5 + 0.5;
    float closestDepth = texture2D(scn_ShadowTexture, shadowPos.xy).r;
    float shadow = (shadowPos.z - bias) > closestDepth ? 0.5 : 0.0;
    return shadow;
}

void main()
{
    vec3 normal = normalize(v_EyeNormal);
    vec3 viewDir = normalize(v_EyeFragPos);

    // directional light
    float bias = max(0.005 * (1.0 - dot(normal, scn_DirLightViewDir)), 0.0005);  
    float shadow = CalculateShadow(v_ShadowFragPos, bias);

    vec3 reflectDir = reflect(-scn_DirLightViewDir, normal);
    vec3 result = CalculateLight(normal, viewDir, scn_DirLightViewDir, reflectDir, shadow) * scn_DirLightColor;

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

        vec3 power = CalculateLight(normal, viewDir, lightDir2, reflectDir2, 0.0) * scn_PointLightColor[i];
        result += max(power * attenuation, vec3(0.0, 0.0, 0.0));
    }

    gl_FragColor = vec4(result, 1.0) * v_Color;
}