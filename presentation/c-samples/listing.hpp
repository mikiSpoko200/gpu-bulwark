#pragma once

#include <memory>
#include <string>
#include <iostream>
#include <stdexcept>
#include <ostream>
#include <vector>

#include <windows.h>
#include "wrapper.hpp"

std::string GetErrorMessage(DWORD errorCode) {
    LPVOID lpMsgBuf;
    DWORD bufLen = FormatMessage(
        FORMAT_MESSAGE_ALLOCATE_BUFFER |
        FORMAT_MESSAGE_FROM_SYSTEM |
        FORMAT_MESSAGE_IGNORE_INSERTS,
        NULL,
        errorCode,
        MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT),
        (LPTSTR)&lpMsgBuf,
        0, NULL);

    if (bufLen) {
        std::wstring msg((LPTSTR)lpMsgBuf, bufLen);
        LocalFree(lpMsgBuf);
        return std::string(msg.begin(), msg.end());
    } else {
        return "Unknown error";
    }
}

std::string ReadShaderFile(const std::wstring& fileName) {
    HANDLE fileHandle = CreateFile(
        fileName.c_str(),
        GENERIC_READ,
        0,
        NULL,
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        NULL
    );

    if (fileHandle == INVALID_HANDLE_VALUE) {
        DWORD errorCode = GetLastError();
        std::string errorMessage = GetErrorMessage(errorCode);

        std::wstring wideErrorMessage = std::wstring(errorMessage.begin(), errorMessage.end());
        std::wstring fullMessage = L"Failed to open file: " + fileName + L"\nError: " + wideErrorMessage;

        MessageBoxW(NULL, fullMessage.c_str(), TEXT("Error"), MB_OK | MB_ICONERROR);

        throw std::runtime_error("Failed to open file: " + errorMessage);
    }

    DWORD fileSize = GetFileSize(fileHandle, NULL);
    if (fileSize == INVALID_FILE_SIZE) {
        DWORD errorCode = GetLastError();
        std::string errorMessage = GetErrorMessage(errorCode);

        CloseHandle(fileHandle);

        std::wstring wideErrorMessage = std::wstring(errorMessage.begin(), errorMessage.end());
        std::wstring fullMessage = L"Failed to get file size for file: " + fileName + L"\nError: " + wideErrorMessage;

        MessageBoxW(NULL, fullMessage.c_str(), TEXT("Error"), MB_OK | MB_ICONERROR);

        throw std::runtime_error("Failed to get file size: " + errorMessage);
    }

    std::string fileContent(fileSize, '\0');
    DWORD bytesRead;
    BOOL success = ReadFile(
        fileHandle,
        &fileContent[0],
        fileSize,
        &bytesRead,
        NULL
    );

    CloseHandle(fileHandle);

    if (!success || bytesRead != fileSize) {
        DWORD errorCode = GetLastError();
        std::string errorMessage = GetErrorMessage(errorCode);

        std::wstring wideErrorMessage = std::wstring(errorMessage.begin(), errorMessage.end());
        std::wstring fullMessage = L"Failed to read file: " + fileName + L"\nError: " + wideErrorMessage;

        MessageBoxW(NULL, fullMessage.c_str(), TEXT("Error"), MB_OK | MB_ICONERROR);

        throw std::runtime_error("Failed to read file: " + errorMessage);
    }

    return fileContent;
}


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
