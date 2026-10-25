@echo off
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\build-installer.ps1"
if errorlevel 1 (
  echo Creazione installer non completata.
) else (
  echo Installer disponibile nella cartella dist.
)
pause
