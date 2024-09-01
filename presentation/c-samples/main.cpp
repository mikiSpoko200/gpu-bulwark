#include <string>
#include <iostream>
#include <stdexcept>
#include <ostream>
#include <vector>

#include <windows.h>
#include "wrapper.hpp"

#define internal static

LRESULT CALLBACK WindowProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam);
void SetupOpenGL(HWND hwnd);
void InitializeGL();
void Render();

internal HWND hwnd;

struct GL {
    VertexArray vao = VertexArray();
    Buffer<float> colorBuffer = Buffer<float>::Array();
    Buffer<float> positionBuffer = Buffer<float>::Array();
    Program program = Program();
};

internal struct GL* Gl;

void ShowCurrentWorkingDirectory() {
    DWORD bufferSize = GetCurrentDirectory(0, NULL);
    if (bufferSize == 0) {
        throw std::runtime_error("Failed to get the current directory size.");
    }

    std::wstring currentDirectory(bufferSize, L'\0');
    DWORD length = GetCurrentDirectory(bufferSize, &currentDirectory[0]);

    if (length == 0) {
        throw std::runtime_error("Failed to get the current directory.");
    }

    MessageBoxW(NULL, currentDirectory.c_str(), L"Current Working Directory", MB_OK | MB_ICONINFORMATION);
}

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

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nCmdShow) {

    WNDCLASS wc = { };
    wc.lpfnWndProc = WindowProc;
    wc.hInstance = hInstance;
    wc.lpszClassName = TEXT("OpenGLWindowClass");
    wc.style = CS_OWNDC;

    RegisterClass(&wc);

    hwnd = CreateWindowEx(
        0,
        TEXT("OpenGLWindowClass"), 
        TEXT("OpenGL Window"), 
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT, CW_USEDEFAULT,
        800, 600,
        NULL,
        NULL,
        hInstance,
        NULL
    );

    if (hwnd == NULL) {
        return 0;
    }

    ShowWindow(hwnd, nCmdShow);

    SetupOpenGL(hwnd);
    InitializeGL();

    MSG msg = {};
    while (GetMessage(&msg, NULL, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);

        Render();
    }

    wglMakeCurrent(NULL, NULL);
    wglDeleteContext(wglGetCurrentContext());
    ReleaseDC(hwnd, GetDC(hwnd));
    DestroyWindow(hwnd);

    return 0;
}

void SetupOpenGL(HWND hwnd) {
    HDC hdc = GetDC(hwnd);

    PIXELFORMATDESCRIPTOR pfd = {};
    pfd.nSize = sizeof(PIXELFORMATDESCRIPTOR);
    pfd.nVersion = 1;
    pfd.dwFlags = PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER;
    pfd.iPixelType = PFD_TYPE_RGBA;
    pfd.cColorBits = 32;
    pfd.cDepthBits = 24;
    pfd.iLayerType = PFD_MAIN_PLANE;

    int pixelFormat = ChoosePixelFormat(hdc, &pfd);
    SetPixelFormat(hdc, pixelFormat, &pfd);

    HGLRC tempContext = wglCreateContext(hdc);
    wglMakeCurrent(hdc, tempContext);

    glewExperimental = TRUE;
    if (glewInit() != GLEW_OK) {
        MessageBox(hwnd, TEXT("Failed to initialize GLEW"), TEXT("Error"), MB_OK | MB_ICONERROR);
        exit(-1);
    }

    int attribs[] = {
        WGL_CONTEXT_MAJOR_VERSION_ARB, 4,
        WGL_CONTEXT_MINOR_VERSION_ARB, 6,
        WGL_CONTEXT_PROFILE_MASK_ARB,  WGL_CONTEXT_CORE_PROFILE_BIT_ARB,
        0
    };

    if (wglewIsSupported("WGL_ARB_create_context")) {
        HGLRC hglrc = wglCreateContextAttribsARB(hdc, 0, attribs);
        wglMakeCurrent(NULL, NULL);
        wglDeleteContext(tempContext);
        wglMakeCurrent(hdc, hglrc);
    } else {
        MessageBox(hwnd, TEXT("OpenGL 4.6 not supported"), TEXT("Error"), MB_OK | MB_ICONERROR);
        exit(-1);
    }

    ReleaseDC(hwnd, hdc);
}

void InitializeGL() {
    Gl = new GL;
    
    {
        std::string vertexShaderCode = ReadShaderFile(L"D:\\dev\\gpu-bulwark\\presentation\\shaders\\hello_vertices.vert");
        std::string fragmentShaderCode = ReadShaderFile(L"D:\\dev\\gpu-bulwark\\presentation\\shaders\\hello_vertices.frag");

        Shader vertexShader = Shader::Vertex(vertexShaderCode);
        Shader fragmentShader = Shader::Fragment(fragmentShaderCode);

        Gl->program.AttachShader(vertexShader);
        Gl->program.AttachShader(fragmentShader);
        Gl->program.Link();
    }

    const std::vector<float> colors = {
        1.0f, 0.0f, 0.0f,
        0.0f, 1.0f, 0.0f,
        0.0f, 0.0f, 1.0f
    };

    const std::vector<float> positions = {
        -0.5f, -0.5f, -1.0f,
         0.5f, -0.5f, -1.0f,
         0.0f,  0.5f, -1.0f
    };

    Gl->vao.Bind();
    
    Gl->colorBuffer.Data(colors, GL_STATIC_DRAW);
    Gl->vao.VertexAttribPointer(0, Gl->colorBuffer, 3, GL_FLOAT);

    Gl->positionBuffer.Data(positions, GL_STATIC_DRAW);
    Gl->vao.VertexAttribPointer(1, Gl->positionBuffer, 3, GL_FLOAT);

    glClearColor(0.3f, 0.4f, 0.7f, 1.0f); CHECK_GL_ERROR;
}

void Render() {
    HDC hdc = GetDC(hwnd);

    Gl->vao.Bind();
    Gl->program.Use();

    glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT); CHECK_GL_ERROR;

    glDrawArrays(GL_TRIANGLES, 0, 3); CHECK_GL_ERROR;

    SwapBuffers(hdc); CHECK_GL_ERROR;
    ReleaseDC(hwnd, hdc);
}

LRESULT CALLBACK WindowProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam) {
    switch (uMsg) {
        case WM_DESTROY:
            PostQuitMessage(0);
            return 0;

        case WM_SIZE: {
            int width = LOWORD(lParam);
            int height = HIWORD(lParam);
            glViewport(0, 0, width, height);
        }
        return 0;
    }

    return DefWindowProc(hwnd, uMsg, wParam, lParam);
}
