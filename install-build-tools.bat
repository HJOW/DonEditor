@echo off
rem Don Editor build prerequisites launcher (GPL-3.0-or-later).
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0script\install-build-tools.ps1" %*
exit /b %ERRORLEVEL%
