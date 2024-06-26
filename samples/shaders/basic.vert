#version 330 core
#extension GL_ARB_explicit_uniform_location: enable

layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;

layout(location = 0) uniform float offset;

out vec4 vertexColor;

vec3 add_one(const vec3 src);
vec3 sub_one(const vec3 src);

void main() {
    vec3 moved = add_one(position);
    moved = sub_one(moved);

    gl_Position = vec4(moved, 1.0) + offset;
    vertexColor = color;
}