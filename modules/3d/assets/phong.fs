#version 100
precision lowp float;

#define MAX_POINT_LIGHTS 4
#define MAX_DIR_LIGHTS 1

varying vec3 v_DirLitShadowPos[MAX_DIR_LIGHTS];
varying vec3 v_EyeFragPos;
varying vec3 v_EyeNormal;
varying vec4 v_Color;

uniform vec3 scn_DirLitViewDir[MAX_DIR_LIGHTS];
uniform vec3 scn_DirLitColor[MAX_DIR_LIGHTS];
uniform sampler2D scn_DirLitShadowTexture[MAX_DIR_LIGHTS];

uniform vec3 scn_PointLitViewPos[MAX_POINT_LIGHTS];
uniform vec3 scn_PointLitColor[MAX_POINT_LIGHTS];
uniform vec3 scn_PointLitAttenuation[MAX_POINT_LIGHTS];

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

float CalculateShadow(sampler2D shadowTexture, vec3 shadowPos, float bias)
{
    // transform to [0,1] range.
    shadowPos = shadowPos * 0.5 + 0.5;
    float closestDepth = texture2D(shadowTexture, shadowPos.xy).r;
    return (shadowPos.z - bias) > closestDepth ? 0.5 : 0.0;
}

void main()
{
    vec3 normal = normalize(v_EyeNormal);
    vec3 viewDir = normalize(v_EyeFragPos);
    vec3 result = vec3(0.0, 0.0, 0.0);

    // directional light
    for(int i = 0; i < MAX_DIR_LIGHTS; i++)
    {
        float bias = max(0.005 * (1.0 - dot(normal, scn_DirLitViewDir[i])), 0.0005);
        float shadow = CalculateShadow(scn_DirLitShadowTexture[i], v_DirLitShadowPos[i], bias);

        vec3 reflectDir = reflect(-scn_DirLitViewDir[i], normal);
        result += CalculateLight(normal, viewDir, scn_DirLitViewDir[i], reflectDir, shadow) * scn_DirLitColor[i];
    }

    // point lights
    for(int i = 0; i < MAX_POINT_LIGHTS; i++)
    {
        vec3 lightDir2 = normalize(v_EyeFragPos - scn_PointLitViewPos[i]);
        vec3 reflectDir2 = reflect(-lightDir2, normal);
        float distance = length(scn_PointLitViewPos[i] - v_EyeFragPos);
        float attenuation =
            scn_PointLitAttenuation[i].x +
            scn_PointLitAttenuation[i].y * distance +
            scn_PointLitAttenuation[i].z * (distance * distance);

        vec3 power = CalculateLight(normal, viewDir, lightDir2, reflectDir2, 0.0) * scn_PointLitColor[i];
        result += max(power * attenuation, vec3(0.0, 0.0, 0.0));
    }

    gl_FragColor = vec4(result, 1.0) * v_Color;
}