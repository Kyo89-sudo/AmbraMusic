@echo off
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\build-windows.ps1"
if errorlevel 1 (
  echo.
  echo Compilazione non completata. Leggi COMPILAZIONE.log.
) else (
  echo.
  echo Programma disponibile nella cartella dist
)
pause
