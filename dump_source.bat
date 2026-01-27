@echo off
cd /d "%~dp0"

echo ========================================================
echo  RustClipboard Translator Source Code Dumper
echo ========================================================
echo.
echo Running PowerShell script (dump_source.ps1)...
echo.

powershell -NoProfile -ExecutionPolicy Bypass -File "dump_source.ps1"

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] Script execution failed with error code %ERRORLEVEL%.
    pause
    exit /b %ERRORLEVEL%
)

echo.
echo ========================================================
echo  Dump Complete!
echo ========================================================
pause
