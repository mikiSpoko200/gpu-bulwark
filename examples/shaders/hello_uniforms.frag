#version 420 core

#extension GL_ARB_explicit_uniform_location: enable
#extension GL_ARB_separate_shader_objects: enable
#extension GL_ARB_shading_language_420pack: enable

layout(location = 0) in vec4 vertexColor;
layout(location = 1) in vec2 uv;

out vec4 FragColor;

void main()
{
    FragColor = vertexColor; //  * vertexColor
}