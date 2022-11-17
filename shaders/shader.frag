#version 450
	
layout(location = 0) out vec4 diffuseColor;

layout (location=0) in vec4 v_color;

void main(){
    diffuseColor = v_color;
}