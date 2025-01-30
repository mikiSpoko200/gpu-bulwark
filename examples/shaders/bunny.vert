#version 460 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 normal;

layout(location = 0) uniform mat4 view_matrix;

layout(location = 0) out vec4 fragNormal;

void main() {
    gl_Position = vec4(position, 1) * view_matrix;
    fragNormal = normal;
}