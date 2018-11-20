varying vec3 v_EyeFragPos;
varying vec3 v_EyeNormal;
varying vec2 v_Texcoord;

uniform vec3 u_DirLitViewDir[MAX_DIR_LITS];
uniform vec3 u_DirLitColor[MAX_DIR_LITS];

uniform vec3 u_PointLitViewPos[MAX_POINT_LITS];
uniform vec3 u_PointLitColor[MAX_POINT_LITS];
uniform vec3 u_PointLitAttenuation[MAX_POINT_LITS];

uniform vec3 u_GlobalAmbient;

uniform vec3 u_Diffuse;
uniform sampler2D u_DiffuseTexture;

uniform vec3 u_Specular;
uniform sampler2D u_SpecularTexture;

uniform float u_Shininess;

vec3 Calculate(vec3 normal, vec3 viewDir, vec3 lightDir, vec3 reflectDir, vec3 d, vec3 s)
{
    vec3 diffuse = max(dot(normal, -lightDir), 0.0) * u_Diffuse * d;
    vec3 specular = pow(max(dot(viewDir, reflectDir), 0.0), u_Shininess) * u_Specular * s;
    return diffuse + specular;
}

void main()
{
    vec3 normal = normalize(v_EyeNormal);
    vec3 viewDir = normalize(v_EyeFragPos);

    vec3 diffuse = texture2D(u_DiffuseTexture, v_Texcoord).rgb;
    vec3 specular = texture2D(u_SpecularTexture, v_Texcoord).rgb;

    //
    vec3 result = u_GlobalAmbient * diffuse;

    // directional light
    for(int i = 0; i < MAX_DIR_LITS; i++)
    {
        // slope-scale depth bias
        // float bias = max(0.005 * (1.0 - dot(normal, u_DirLitViewDir[i])), 0.0005);
        // float shadow = CalculateShadow(scn_DirLitShadowTexture[i], v_DirLitShadowPos[i], bias);
        // result += Calculate(normal, viewDir, u_DirLitViewDir[i], reflectDir, shadow) * u_DirLitColor[i];

        vec3 reflectDir = reflect(-u_DirLitViewDir[i], normal);
        result += Calculate(normal, viewDir, u_DirLitViewDir[i], reflectDir, diffuse, specular) * u_DirLitColor[i];
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

        vec3 power = Calculate(normal, viewDir, lightDir2, reflectDir2, diffuse, specular) * u_PointLitColor[i];
        result += max(power * attenuation, vec3(0.0, 0.0, 0.0));
    }

    gl_FragColor = vec4(result, 1.0);
}