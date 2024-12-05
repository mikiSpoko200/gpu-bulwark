#include <string>
#include <iostream>
#include <stdexcept>
#include <ostream>
#include <vector>

#define GLEW_STATIC

#include "deps/GL/glew.h"
#include <windows.h>
#include "wrapper.hpp"
#include "listing.hpp"

LRESULT CALLBACK WindowProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam);

class App {
    Listing* listing;
    HWND hwnd;
    HDC hdc;
    HGLRC hglrc;
public:
    App(HWND hwnd) : hwnd(hwnd) {
        hdc = GetDC(hwnd);

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
            hglrc = wglCreateContextAttribsARB(hdc, 0, attribs);
            wglMakeCurrent(NULL, NULL);
            wglDeleteContext(tempContext);
            wglMakeCurrent(hdc, hglrc);
        } else {
            MessageBox(hwnd, TEXT("OpenGL 4.6 not supported"), TEXT("Error"), MB_OK | MB_ICONERROR);
            exit(-1);
        }

        listing = new Listing();
        glClearColor(0.4f, 0.5f, 0.6f, 1.0f); CHECK_GL_ERROR;
    }

    void Render() const {
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT); CHECK_GL_ERROR;

        listing->Render();

        glDrawArrays(GL_TRIANGLES, 0, 3); CHECK_GL_ERROR;

        SwapBuffers(hdc);
    }

    ~App() {
        wglMakeCurrent(NULL, NULL);
        wglDeleteContext(hglrc);
        ReleaseDC(hwnd, hdc);
        delete listing;
        DestroyWindow(hwnd);
    }
};

int WINAPI WinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPSTR lpCmdLine, int nCmdShow) {
    WNDCLASS wc = { };
    wc.lpfnWndProc = WindowProc;
    wc.hInstance = hInstance;
    wc.lpszClassName = TEXT("OpenGLWindowClass");
    wc.style = CS_OWNDC;

    RegisterClass(&wc);

    HWND hwnd = CreateWindowEx(
        0,
        TEXT("OpenGLWindowClass"), 
        TEXT("OpenGL Window"), 
        WS_OVERLAPPEDWINDOW & ~WS_SIZEBOX & ~WS_MAXIMIZEBOX & ~WS_MINIMIZEBOX,
        CW_USEDEFAULT, CW_USEDEFAULT,
        800, 600,
        NULL,
        NULL,
        hInstance,
        NULL
    );

    if (hwnd == NULL) { return 0; }

    ShowWindow(hwnd, nCmdShow);

    auto app = App(hwnd);

    MSG msg = { };
    while (GetMessage(&msg, NULL, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);

        app.Render();
    }

    return 0;
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
