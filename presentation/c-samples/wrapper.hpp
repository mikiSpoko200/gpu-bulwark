#pragma once

#include <string>
#include <vector>
#include <array>
#include <GL/glew.h>
#include <GL/wglew.h>
#include <GL/gl.h>
#include <windows.h>

// #define RT_CHECK

#ifdef RT_CHECK
#define CHECK_GL_ERROR \
    { \
        GLenum err; \
        while ((err = glGetError()) != GL_NO_ERROR) { \
            std::string errorMessage = "OpenGL error in file " + std::string(__FILE__) + \
                                       " at line " + std::to_string(__LINE__) + ": " + \
                                       GetGLErrorString(err); \
            MessageBoxA(NULL, errorMessage.c_str(), "OpenGL Error", MB_OK | MB_ICONERROR); \
            exit(-1); \
        } \
    }
#else
#define CHECK_GL_ERROR
#endif

const char* GetGLErrorString(GLenum err) {
    switch (err) {
        case GL_NO_ERROR:                       return "No error";
        case GL_INVALID_ENUM:                   return "Invalid enum";
        case GL_INVALID_VALUE:                  return "Invalid value";
        case GL_INVALID_OPERATION:              return "Invalid operation";
        case GL_STACK_OVERFLOW:                 return "Stack overflow";
        case GL_STACK_UNDERFLOW:                return "Stack underflow";
        case GL_OUT_OF_MEMORY:                  return "Out of memory";
        case GL_INVALID_FRAMEBUFFER_OPERATION:  return "Invalid framebuffer operation";
        default:                                return "Unknown error";
    }
}


class Object {
protected:
    GLuint name;

public:
    Object() : name(0) { }

    GLuint Name() const {
        return name;
    }
};

template<class T>
class Buffer : public Object {
    GLenum target;

public:
    explicit Buffer(GLenum target) : target(target) {
        glCreateBuffers(1, &name); CHECK_GL_ERROR;
    }

    void Bind() const {
        glBindBuffer(target, name); CHECK_GL_ERROR;
    }

    void Unbind() const {
        glBindBuffer(target, 0); CHECK_GL_ERROR;
    }

    void Data(const std::vector<T>& data, GLenum usage) {
        Bind();
        glBufferData(target, sizeof(T) * data.size(), data.data(), usage); CHECK_GL_ERROR;
        Unbind();
    }

    static Buffer<T> Array() {
        return Buffer<T>(GL_ARRAY_BUFFER);
    }
};

class VertexArray : public Object {
public:
    VertexArray() {
        glCreateVertexArrays(1, &name); CHECK_GL_ERROR;
    }

    void Bind() const {
        glBindVertexArray(name); CHECK_GL_ERROR;
    }

    void Unbind() const {
        glBindVertexArray(0); CHECK_GL_ERROR;
    }

    template <class T>
    void VertexAttribPointer(GLuint index, const Buffer<T>& buffer, GLint size, GLenum type) const {
        Bind();
        buffer.Bind();
        glVertexAttribPointer(index, size, type, GL_FALSE, size * sizeof(T), (void*)0); CHECK_GL_ERROR;
        glEnableVertexAttribArray(index); CHECK_GL_ERROR;
        buffer.Unbind();
        Unbind();
    }
};

class Shader : public Object {
    GLenum target;

public:
    Shader(GLenum target, const std::string& source) : target(target) {
        name = glCreateShader(target); CHECK_GL_ERROR;

        const char* src = source.c_str();
        glShaderSource(name, 1, &src, nullptr); CHECK_GL_ERROR;

        glCompileShader(name); CHECK_GL_ERROR;

        GLint success;
        glGetShaderiv(name, GL_COMPILE_STATUS, &success); CHECK_GL_ERROR;

        if (!success) {
            char infoLog[512];
            glGetShaderInfoLog(name, 512, nullptr, infoLog); CHECK_GL_ERROR;
            MessageBoxA(nullptr, infoLog, "Shader Compilation Error", MB_OK | MB_ICONERROR);
            exit(-1);
        }
    }

    ~Shader() {
        glDeleteShader(name); CHECK_GL_ERROR;
    }

    static Shader Vertex(const std::string& source) {
        return Shader(GL_VERTEX_SHADER, source);
    }

    static Shader Fragment(const std::string& source) {
        return Shader(GL_FRAGMENT_SHADER, source);
    }
};

class Program : public Object {
public:
    Program() {
        name = glCreateProgram(); CHECK_GL_ERROR;
    }

    void AttachShader(const Shader& shader) const {
        glAttachShader(name, shader.Name()); CHECK_GL_ERROR;
    }

    void Link() const {
        glLinkProgram(name); CHECK_GL_ERROR;

        GLint success;
        glGetProgramiv(name, GL_LINK_STATUS, &success); CHECK_GL_ERROR;

        if (!success) {
            char infoLog[512];
            glGetProgramInfoLog(name, 512, nullptr, infoLog); CHECK_GL_ERROR;
            MessageBoxA(nullptr, infoLog, "Program Linking Error", MB_OK | MB_ICONERROR);
            exit(-1);
        }
    }

    void Use() const {
        glUseProgram(name); CHECK_GL_ERROR;
    }

    ~Program() {
        glDeleteProgram(name); CHECK_GL_ERROR;
    }
};

class Texture : public Object {
    GLenum target;

public:
    explicit Texture(GLenum target) : target(target) {
        glGenTextures(1, &name); CHECK_GL_ERROR;
    }

    void Bind() const {
        glBindTexture(target, name); CHECK_GL_ERROR;
    }

    void Unbind() const {
        glBindTexture(target, 0); CHECK_GL_ERROR;
    }

    void Storage2D(GLenum internalFormat, GLsizei width, GLsizei height) const {
        Bind();
        glTexStorage2D(target, 1, internalFormat, width, height);
        Unbind();
    }

    void SubImage2D(GLint xoffset, GLint yoffset, GLsizei width, GLsizei height, GLenum format, GLenum type, const void* data) const {
        Bind();
        glTexSubImage2D(target, 0, xoffset, yoffset, width, height, format, type, data); CHECK_GL_ERROR;
        Unbind();
    }

    ~Texture() {
        glDeleteTextures(1, &name); CHECK_GL_ERROR;
    }

    static Texture CreateWithStorage2D() {
        return Texture(GL_TEXTURE_2D);
    }
};
