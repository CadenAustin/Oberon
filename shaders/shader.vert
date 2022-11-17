#version 450

layout (location=0) in vec4 position;
layout (location=1) in float size;
layout (location=2) in vec4 color;

layout (location=0) out vec4 v_color;

void main() {
    gl_PointSize = size;
    gl_Position = position;
    v_color = color;
}