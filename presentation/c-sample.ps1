Write-Host "Building C sample . . ."
msbuild c-samples/c-samples.sln /p:Configuration=Release /p:Platform=x64 /nologo /verbosity:quiet

Read-Host -Prompt "Press Enter run C sample . . ."
.\c-samples\x64\Release\c-samples.exe
