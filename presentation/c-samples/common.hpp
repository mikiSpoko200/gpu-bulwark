#pragma once

#include <string>
#include <stdexcept>

#include <windows.h>

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
    }
    else {
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