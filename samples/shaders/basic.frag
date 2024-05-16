#version 330 core

#extension GL_ARB_explicit_uniform_location: enable
#extension GL_ARB_separate_shader_objects: enable
#extension GL_ARB_shading_language_420pack: enable

layout(location = 0) in vec4 vertexColor;
layout(location = 1) in vec2 uv;
layout(location = 8) uniform sampler2D fancyTexture;

out vec4 FragColor;

void main()
{
    FragColor = texture(fancyTexture, uv); //  * vertexColor
}