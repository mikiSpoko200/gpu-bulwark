#version 460 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;

layout(location = 0) uniform float multiplier;
layout(location = 1) uniform float x_offset;
layout(location = 2) uniform float y_offset;

layout(location = 0) out vec4 vertexColor;

void main() {
    gl_Position = vec4(position, 1.0);
    gl_Position.x += x_offset;
    gl_Position.y += y_offset;
    vertexColor = color * multiplier;
}
