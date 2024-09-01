#include "sample.hpp"
#include "wrapper.hpp"

#include "hello_triangle.hpp"

class HelloTriangle : Sample {
    Program program = Program();
    VertexArray vao = VertexArray();

public:
    void Render() final override {
        std::string vertexShaderCode = ReadShaderFile(L"D:\\dev\\gpu-bulwark\\presentation\\shaders\\hello_vertices.vert");
        std::string fragmentShaderCode = ReadShaderFile(L"D:\\dev\\gpu-bulwark\\presentation\\shaders\\hello_vertices.frag");
    }
};
