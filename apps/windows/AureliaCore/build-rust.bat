@echo off
setlocal enabledelayedexpansion

REM Build Rust aurelia-core for Windows and generate C# UniFFI bindings.
REM Usage: build-rust.bat [--release]

set "SCRIPT_DIR=%~dp0"
set "PROJECT_ROOT=%SCRIPT_DIR%..\..\.."
set "CORE_CRATE=aurelia-core"

REM Remove trailing backslash from SCRIPT_DIR
set "OUT_DIR=%SCRIPT_DIR%"
if "%OUT_DIR:~-1%"=="\" set "OUT_DIR=%OUT_DIR:~0,-1%"

set "TEMP_DIR=%OUT_DIR%\temp_bindgen"

set "PROFILE=debug"
set "CARGO_FLAGS="

if "%~1"=="--release" (
    set "PROFILE=release"
    set "CARGO_FLAGS=--release"
)

set "TARGET_DIR=%PROJECT_ROOT%\target"
set "DLL_PATH=%TARGET_DIR%\%PROFILE%\aurelia_core.dll"

echo Building aurelia-core for Windows...
cd /d "%PROJECT_ROOT%"
cargo build -p %CORE_CRATE% %CARGO_FLAGS% --features desktop

if errorlevel 1 (
    echo Build failed!
    exit /b 1
)

REM Create temp directory
if not exist "%TEMP_DIR%" mkdir "%TEMP_DIR%"

echo Generating C# UniFFI bindings...

REM First pass: generate core types (AureliaCore namespace)
cargo run -p uniffi-bindgen -- generate ^
    --library "%DLL_PATH%" ^
    --language csharp ^
    --config "%OUT_DIR%\uniffi.toml" ^
    --out-dir "%TEMP_DIR%" ^
    --no-format

if errorlevel 1 (
    echo Binding generation failed for aurelia-core!
    exit /b 1
)

REM Copy core bindings
copy /Y "%TEMP_DIR%\aurelia_core.cs" "%OUT_DIR%\aurelia_core.cs"

REM Second pass: generate lyrics types (aurelia_lyrics namespace)
cargo run -p uniffi-bindgen -- generate ^
    --library "%DLL_PATH%" ^
    --language csharp ^
    --config "%OUT_DIR%\uniffi.lyrics.toml" ^
    --out-dir "%TEMP_DIR%" ^
    --no-format

if errorlevel 1 (
    echo Binding generation failed for aurelia-lyrics!
    exit /b 1
)

REM Copy lyrics bindings
copy /Y "%TEMP_DIR%\aurelia_lyrics.cs" "%OUT_DIR%\aurelia_lyrics.cs"

REM Clean up temp directory
rmdir /s /q "%TEMP_DIR%"

echo Done!
