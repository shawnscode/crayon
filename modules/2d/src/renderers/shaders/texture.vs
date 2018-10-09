#version 100
precision lowp float;

attribute vec2 Position;
attribute vec2 Texcoord0;
attribute vec4 Color0;
attribute vec4 Color1;

varying vec2 v_Texcoord;
varying vec4 v_Color;
varying vec4 v_AdditiveColor;

void main(){
    gl_Position = vec4(Position, 0.0, 1.0);

    v_Texcoord = Texcoord0;
    v_Color = Color0;
    v_AdditiveColor = Color1;
}
