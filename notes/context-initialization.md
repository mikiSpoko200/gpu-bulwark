# Context Initialization

## Windows

We use `GetDC` for Device Context which is the used in `wglCreateContext` and then subsequently bind with `wglMakeCurrent`.

Beforehand though due to legacy features we need to set pixel format with combination of `ChoosePixelFormat` to get system index for derired format of the framebuffer, `DescribePixelFormat` to fill structure with more details and finally make windows use it with `SetPixelFormat`.