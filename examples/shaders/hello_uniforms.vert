#version 460 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;

layout(location = 0) uniform mat4 view_matrix;
layout(location = 1) uniform float scale;

layout(location = 0) out vec4 vertexColor;

vec3 add_one(const vec3 src);
vec3 sub_one(const vec3 src);

void main() {
    vec3 moved = add_one(position);
    moved = sub_one(moved);

    gl_Position = vec4(moved, 1.0) * view_matrix * scale;
    vertexColor = color;
}