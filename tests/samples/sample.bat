@echo off
rem 한 줄 주석
:: 다른 형태의 주석
setlocal enabledelayedexpansion

set NAME=Don Editor
set /a COUNT=0

:loop
if %COUNT% geq 3 goto done
echo 반복 !COUNT! %NAME%
set /a COUNT+=1
goto loop

:done
for %%f in (*.txt) do (
    echo 파일: %%f
)
call :show "인자"
exit /b 0

:show
echo %~1
goto :eof
