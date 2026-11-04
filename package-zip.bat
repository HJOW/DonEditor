@echo off
rem Don Editor release zip builder (wrapper for script\package-zip.ps1).
rem Runs the PowerShell script with ExecutionPolicy Bypass and forwards all arguments.
rem   Example: package-zip.bat -Yes -IncludeSource      Help: package-zip.bat -Help
rem This file is intentionally ASCII-only: cmd.exe misparses UTF-8 batch files after "chcp 65001".
rem All Korean messages are printed by the PowerShell script itself.
setlocal

powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0script\package-zip.ps1" %*
set "EXIT_CODE=%ERRORLEVEL%"

if not "%EXIT_CODE%"=="0" echo.
if not "%EXIT_CODE%"=="0" echo [package-zip] failed with exit code %EXIT_CODE%.

rem Pause only when started by double-click, so the window does not close immediately.
rem The Windows find.exe is named explicitly because a Unix find may come first in PATH.
echo %cmdcmdline% | "%SystemRoot%\System32\find.exe" /i "%~nx0" >nul
if not errorlevel 1 pause

endlocal & exit /b %EXIT_CODE%
