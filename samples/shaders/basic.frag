#version 330 core
in vec4 vertexColor;

out vec4 FragColor;

void foo()
{
    FragColor = vec4(1.0, 1.0, 1.0, 1.0); //vertexColor;
}