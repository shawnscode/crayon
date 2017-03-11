#version 150
in vec2 Position;
out vec2 UV;
void main(){
    gl_Position = vec4(Position, 0.0, 1.0);
    UV = (Position+vec2(1,1))/2.0;
}
