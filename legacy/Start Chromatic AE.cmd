@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0Start-Chromatic.ps1"
if errorlevel 1 pause
