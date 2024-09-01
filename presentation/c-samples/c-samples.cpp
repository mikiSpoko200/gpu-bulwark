#include <string>
#include <iostream>
#include <stdexcept>
#include <ostream>
#include <vector>

#include <windows.h>
#include "wrapper.hpp"

#define internal static

// Function Declarations
LRESULT CALLBACK WindowProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam);
void SetupOpenGL(HWND hwnd);
void InitializeGL();
void Render();

// Global Variables
internal HWND hwnd;
internal struct {
    VertexArray vao = VertexArray();
    Buffer<float> colorBuffer = Buffer<float>::Array();
    Buffer<float> positionBuffer = Buffer<float>::Array();
    Program program = Program();
} GL;

const char* GetGLErrorString(GLenum err) {
    switch (err) {
        case GL_NO_ERROR:          return "No error";
        case GL_INVALID_ENUM:      return "Invalid enum";
        case GL_INVALID_VALUE:     return "Invalid value";
        case GL_INVALID_OPERATION: return "Invalid operation";
        case GL_STACK_OVERFLOW:    return "Stack overflow";
        case GL_STACK_UNDERFLOW:   return "Stack underflow";
        case GL_OUT_OF_MEMORY:     return "Out of memory";
        case GL_INVALID_FRAMEBUFFER_OPERATION: return "Invalid framebuffer operation";
        default:                   return "Unknown error";
    }
}

void ShowCurrentWorkingDirectory() {
    // Step 1: Get the current working directory
    DWORD bufferSize = GetCurrentDirectory(0, NULL);  // Get the size of the buffer needed
    if (bufferSize == 0) {
        throw std::runtime_error("Failed to get the current directory size.");
    }

    std::wstring currentDirectory(bufferSize, L'\0'); // Create a buffer of the correct size
    DWORD length = GetCurrentDirectory(bufferSize, &currentDirectory[0]);

    if (length == 0) {
        throw std::runtime_error("Failed to get the current directory.");
    }

    // Step 2: Display the current directory in a MessageBox
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
    // Step 1: Open the file
    HANDLE fileHandle = CreateFile(
        fileName.c_str(),          // File name
        GENERIC_READ,              // Open for reading
        0,                         // Do not share
        NULL,                      // Default security
        OPEN_EXISTING,             // Open existing file only
        FILE_ATTRIBUTE_NORMAL,     // Normal file
        NULL                       // No template file
    );

    if (fileHandle == INVALID_HANDLE_VALUE) {
        DWORD errorCode = GetLastError();
        std::string errorMessage = GetErrorMessage(errorCode);

        // Convert the file name and error message to wide strings for MessageBoxW
        std::wstring wideErrorMessage = std::wstring(errorMessage.begin(), errorMessage.end());
        std::wstring fullMessage = L"Failed to open file: " + fileName + L"\nError: " + wideErrorMessage;

        // Display the error in a MessageBox
        MessageBoxW(NULL, fullMessage.c_str(), TEXT("Error"), MB_OK | MB_ICONERROR);
        
        throw std::runtime_error("Failed to open file: " + errorMessage);
    }

    // Step 2: Get the file size
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

    // Step 3: Read the file content into a buffer
    std::string fileContent(fileSize, '\0');
    DWORD bytesRead;
    BOOL success = ReadFile(
        fileHandle,                // Handle to the file
        &fileContent[0],           // Buffer to receive data
        fileSize,                  // Number of bytes to read
        &bytesRead,                // Number of bytes read
        NULL                       // No overlapped structure
    );

    CloseHandle(fileHandle); // Step 4: Close the file handle

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
        0,                              // Optional window styles
        TEXT("OpenGLWindowClass"),      // Window class name
        TEXT("OpenGL Window"),          // Window title
        WS_OVERLAPPEDWINDOW,            // Window style
        CW_USEDEFAULT, CW_USEDEFAULT,   // Position
        800, 600,                       // Size
        NULL,                           // Parent window    
        NULL,                           // Menu
        hInstance,                      // Instance handle
        NULL                            // Additional application data
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

    // Initialize GLEW to set up OpenGL extensions
    glewExperimental = TRUE;
    if (glewInit() != GLEW_OK) {
        MessageBox(hwnd, TEXT("Failed to initialize GLEW"), TEXT("Error"), MB_OK | MB_ICONERROR);
        exit(-1);
    }

    // Set up a modern OpenGL context (3.3)
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
    {
        std::string vertexShaderCode = ReadShaderFile(L"D:\\dev\\gpu-bulwark\\presentation\\shaders\\hello_vertices.vert");
        std::string fragmentShaderCode = ReadShaderFile(L"D:\\dev\\gpu-bulwark\\presentation\\shaders\\hello_vertices.frag");

        Shader vertexShader = Shader::Vertex(vertexShaderCode);
        Shader fragmentShader = Shader::Fragment(fragmentShaderCode);

        // Create program and attach shaders
        GL.program.AttachShader(vertexShader);
        GL.program.AttachShader(fragmentShader);
        GL.program.Link();
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

    GL.vao.Bind();

    GL.colorBuffer.Data(colors, GL_STATIC_DRAW);
    GL.vao.VertexAttribPointer(0, GL.colorBuffer, 3, GL_FLOAT);

    GL.positionBuffer.Data(positions, GL_STATIC_DRAW);
    GL.vao.VertexAttribPointer(1, GL.positionBuffer, 3, GL_FLOAT);

    glClearColor(0.3, 0.4, 0.7, 1.0); CHECK_GL_ERROR;
}

void Render() {
    HDC hdc = GetDC(hwnd);

    GL.vao.Bind();
    GL.program.Use();

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
