@echo off
REM Optional: build-release.bat <path\to\defaults.toml>
REM Embeds the given TOML as the first-run default config (see
REM src-tauri/resources/default-config.example.toml).
if not "%~1"=="" set DEEPNOTIFIER_DEFAULT_CONFIG=%~f1
set PATH=%USERPROFILE%\.cargo\bin;%PATH%
call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64
cd /d E:\GoProject\deepNotifier
npm run tauri build
