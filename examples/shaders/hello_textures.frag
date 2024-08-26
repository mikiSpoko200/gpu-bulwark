#version 420 core

layout(location = 0) in vec2 uv;

out vec4 outColor;

layout(binding = 0) uniform sampler2D tex;

void main() {
    outColor = texture(tex, uv);
    // outColor = vec4(uv.x, 0.0, uv.y, 1.0);
}
