#version 460 core

layout(location = 0) in vec3 frag_normal;
layout(location = 1) in vec3 global_light_dir;

out vec4 frag_color;

void main() {
    frag_color = vec4(dot(global_light_dir, frag_normal));
}
