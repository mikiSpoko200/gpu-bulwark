#version 460 core

layout(location = 0) in vec2 uv;

out vec4 outColor;

// The texture sampler
layout(binding = 0) uniform sampler2D tex;

void main()
{
    outColor = texture(tex, uv);
}
