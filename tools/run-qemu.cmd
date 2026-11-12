@echo off
setlocal
set "BASH=%ProgramFiles%\Git\bin\bash.exe"
if not exist "%BASH%" set "BASH=%LOCALAPPDATA%\Programs\Git\bin\bash.exe"
if not exist "%BASH%" set "BASH=bash"
"%BASH%" "%~dp0run-qemu.sh" %*
exit /b %errorlevel%
