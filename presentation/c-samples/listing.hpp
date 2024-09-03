#pragma once

// #define RT_CHECK

#include "wrapper.hpp"
#include "common.hpp"

class Listing {
    Program program = Program();
    VertexArray vao = VertexArray();
    Buffer<float> colorBuffer = Buffer<float>::Array();
    Buffer<float> positionBuffer = Buffer<float>::Array();
    float attenuation = 1.0f;
    float x_offset = 0.0f;
    float y_offset = 0.0f;
    float color_shift = -1.0f;


public:
    Listing() {
        std::string vertexShaderCode = ReadShaderFile(L"D:\\dev\\gpu-bulwark\\presentation\\shaders\\vert.glsl");
        std::string fragmentShaderCode = ReadShaderFile(L"D:\\dev\\gpu-bulwark\\presentation\\shaders\\frag.glsl");

        Shader vertexShader = Shader::Vertex(vertexShaderCode);
        Shader fragmentShader = Shader::Fragment(fragmentShaderCode);

        program.AttachShader(vertexShader);
        program.AttachShader(fragmentShader);
        program.Link();

        program.Use();

        // NOTE: Location skipped
        glUniform1f(0, 1.0f);
        glUniform1f(1, 0.0f);

        const std::vector<float> colors = {
            1.0f, 0.0f, 0.0f, 1.0f, 
            0.0f, 1.0f, 0.0f, 1.0f, 
            0.0f, 0.0f, 1.0f, 1.0f,
        };

        const std::vector<float> positions = {
            -0.5f, -0.5f, -1.0f,
             0.5f, -0.5f, -1.0f,
             0.0f,  0.5f, -1.0f
        };

        vao.Bind();

        colorBuffer.Data(positions, GL_STATIC_DRAW);
        vao.VertexAttribPointer(0, colorBuffer, 3, GL_FLOAT);

        positionBuffer.Data(colors, GL_STATIC_DRAW);
        vao.VertexAttribPointer(1, positionBuffer, 4, GL_FLOAT);
        
        vao.Bind();
    }

    void Render() {
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT); CHECK_GL_ERROR;

        if (attenuation > 1.0f || attenuation < 0.0f) { 
            color_shift *= -1.0f;
        }
        attenuation = attenuation + color_shift * 0.0005;
        x_offset = x_offset < 1.0f ? x_offset + 0.0005f : -1.0f;
        y_offset = y_offset < 1.0f ? y_offset + 0.0005f : -1.0f;

        glUniform1f(0, attenuation); CHECK_GL_ERROR;
        glUniform1f(1, x_offset); CHECK_GL_ERROR;

        glDrawArrays(GL_TRIANGLES, 0, 3); CHECK_GL_ERROR;
    }
};
