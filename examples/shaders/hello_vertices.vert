#version 460 core

layout(location = 0) in vec3 in_color;
layout(location = 1) in vec3 in_position;

layout(location = 0) out vec4 color;

void main() {
    gl_Position = vec4(in_position, 1.0);
    color = vec4(in_color, 1.0);
}
