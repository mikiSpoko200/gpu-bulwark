#version 460 core

layout(location = 0) in vec2 inPos;
layout(location = 1) in vec2 inUV;

layout(location = 0) out vec2 fragUV;

void main()
{
    fragUV = inUV;
    gl_Position = vec4(inPos, 0.5, 1.0);
}
