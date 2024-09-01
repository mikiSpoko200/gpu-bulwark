#pragma once

class HelloTriangle : public Sample {
    Program program = Program();
    VertexArray vao = VertexArray();

public:
    HelloTriangle() {
        std::string vertexShaderCode = ReadShaderFile(L"D:\\dev\\gpu-bulwark\\presentation\\shaders\\hello_triangle.vert");
        std::string fragmentShaderCode = ReadShaderFile(L"D:\\dev\\gpu-bulwark\\presentation\\shaders\\hello_triangle.frag");

        Shader vertexShader = Shader::Vertex(vertexShaderCode);
        Shader fragmentShader = Shader::Fragment(fragmentShaderCode);

        program.AttachShader(vertexShader);
        program.AttachShader(fragmentShader);
        program.Link();

        program.Use();
    }

    void Render() final override {
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT); CHECK_GL_ERROR;
        glDrawArrays(GL_TRIANGLES, 0, 3); CHECK_GL_ERROR;
    }
};
