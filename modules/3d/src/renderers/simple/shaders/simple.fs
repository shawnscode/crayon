varying vec3 v_EyeFragPos;
varying vec3 v_EyeNormal;
varying vec2 v_Texcoord;

uniform vec3 u_DirLitViewDir[MAX_DIR_LITS];
uniform vec3 u_DirLitColor[MAX_DIR_LITS];

uniform vec3 u_PointLitViewPos[MAX_POINT_LITS];
uniform vec3 u_PointLitColor[MAX_POINT_LITS];
uniform vec3 u_PointLitAttenuation[MAX_POINT_LITS];

uniform vec3 u_Ambient;
uniform vec3 u_Diffuse;
uniform vec3 u_Specular;
uniform float u_Shininess;
uniform sampler2D u_DiffuseTexture;

vec3 CalculateLight(vec3 normal, vec3 viewDir, vec3 lightDir, vec3 reflectDir, float shadow)
{
    vec3 diffuse = max(dot(normal, -lightDir), 0.0) * u_Diffuse;
    vec3 specular = pow(max(dot(viewDir, reflectDir), 0.0), u_Shininess) * u_Specular;
    return (1.0 - shadow) * (0.5 * diffuse + specular);
}

// float CalculateShadow(sampler2D shadowTexture, vec3 shadowPos, float bias)
// {
//     // transform to [0, 1] range.
//     shadowPos = shadowPos * 0.5 + 0.5;
//     float closestDepth = texture2D(shadowTexture, shadowPos.xy).r;
//     return (shadowPos.z - bias) > closestDepth ? 0.5 : 0.0;
// }

void main()
{
    vec3 normal = normalize(v_EyeNormal);
    vec3 viewDir = normalize(v_EyeFragPos);
    vec3 result = 0.2 * u_Ambient;

    // directional light
    for(int i = 0; i < MAX_DIR_LITS; i++)
    {
        // slope-scale depth bias
        float bias = max(0.005 * (1.0 - dot(normal, u_DirLitViewDir[i])), 0.0005);
        // float shadow = CalculateShadow(scn_DirLitShadowTexture[i], v_DirLitShadowPos[i], bias);

        vec3 reflectDir = reflect(-u_DirLitViewDir[i], normal);
        // result += CalculateLight(normal, viewDir, u_DirLitViewDir[i], reflectDir, shadow) * u_DirLitColor[i];
        result += CalculateLight(normal, viewDir, u_DirLitViewDir[i], reflectDir, 0.0) * u_DirLitColor[i];
    }

    // point lights
    for(int i = 0; i < MAX_POINT_LITS; i++)
    {
        vec3 lightDir2 = normalize(v_EyeFragPos - u_PointLitViewPos[i]);
        vec3 reflectDir2 = reflect(-lightDir2, normal);
        float distance = length(u_PointLitViewPos[i] - v_EyeFragPos);
        float attenuation =
            u_PointLitAttenuation[i].x +
            u_PointLitAttenuation[i].y * distance +
            u_PointLitAttenuation[i].z * (distance * distance);

        vec3 power = CalculateLight(normal, viewDir, lightDir2, reflectDir2, 0.0) * u_PointLitColor[i];
        result += max(power * attenuation, vec3(0.0, 0.0, 0.0));
    }

    vec4 tex = texture2D(u_DiffuseTexture, v_Texcoord);
    gl_FragColor = vec4(result, 1.0) * tex;
}