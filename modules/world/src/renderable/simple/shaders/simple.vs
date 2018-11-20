attribute vec3 Position;
attribute vec3 Normal;
attribute vec2 Texcoord0;

uniform mat4 u_ModelViewMatrix;
uniform mat4 u_MVPMatrix;
uniform mat4 u_ViewNormalMatrix;

varying vec3 v_EyeFragPos;
varying vec3 v_EyeNormal;
varying vec2 v_Texcoord;

void main() {
    gl_Position = u_MVPMatrix * vec4(Position, 1.0);

    vec4 eyePos = u_ModelViewMatrix * vec4(Position, 1.0);
    v_EyeFragPos = eyePos.xyz / eyePos.w;
    v_EyeNormal = vec3(u_ViewNormalMatrix * vec4(Normal, 0.0));
    v_Texcoord = Texcoord0;
}