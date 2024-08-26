#version 420 core


layout(location = 0) in vec4 vertexColor;

out vec4 FragColor;

void main()
{
    FragColor = vertexColor;
}