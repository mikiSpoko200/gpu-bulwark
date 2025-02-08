#version 460 core

layout(location = 0) in vec4 frag_normal;

layout(location = 1) uniform vec3 global_light_dir;

out vec4 frag_color;

void main() {
    frag_color = normalize(vec4(global_light_dir, 1) * frag_normal);
}
