#pragma once

#include "wrapper.hpp"
#include "common.hpp"

class Listing {
    Program program = Program();
    VertexArray vao = VertexArray();
    Buffer<float> colorBuffer = Buffer<float>::Array();
    Buffer<float> positionBuffer = Buffer<float>::Array();

public:
    Listing() {
        std::string vertexShaderCode = ReadShaderFile(L"D:\\dev\\gpu-bulwark\\presentation\\shaders\\hello_vertices.vert");
        std::string fragmentShaderCode = ReadShaderFile(L"D:\\dev\\gpu-bulwark\\presentation\\shaders\\hello_vertices.frag");

        Shader vertexShader = Shader::Vertex(vertexShaderCode);
        Shader fragmentShader = Shader::Fragment(fragmentShaderCode);

        program.AttachShader(vertexShader);
        program.AttachShader(fragmentShader);
        program.Link();

        // NOTE: shader expects vec3
        const std::vector<float> colors = {
            1.0f, 0.0f, 0.0f,
            0.0f, 1.0f, 0.0f,
            0.0f, 0.0f, 1.0f,
        };

        const std::vector<float> positions = {
            -0.5f, -0.5f, -1.0f,
             0.5f, -0.5f, -1.0f,
             0.0f,  0.5f, -1.0f
        };

        vao.Bind();

        colorBuffer.Data(colors, GL_STATIC_DRAW);
        vao.VertexAttribPointer(0, colorBuffer, 3, GL_FLOAT);

        positionBuffer.Data(positions, GL_STATIC_DRAW);
        vao.VertexAttribPointer(1, positionBuffer, 3, GL_FLOAT);
    }

    void Render() const {
        vao.Bind();
        program.Use();

        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT); CHECK_GL_ERROR;

        glDrawArrays(GL_TRIANGLES, 0, 3); CHECK_GL_ERROR;
    }
};
