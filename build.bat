@echo off
chcp 65001 >nul

REM Agent Playground - Windows Build Script
REM Usage: build.bat [dev|release|clean|help]

setlocal EnableDelayedExpansion

set "PROJECT_ROOT=%~dp0"
set "WEB_DIR=%PROJECT_ROOT%web"
set "STATIC_DIR=%PROJECT_ROOT%crates\api\static"

echo.
echo  🚀 Agent Playground Build Script

echo.

if "%~1"=="" goto :dev
if /I "%~1"=="dev" goto :dev
if /I "%~1"=="release" goto :release
if /I "%~1"=="frontend" goto :frontend
if /I "%~1"=="backend" goto :backend
if /I "%~1"=="clean" goto :clean
if /I "%~1"=="run" goto :run
if /I "%~1"=="help" goto :help

echo [ERROR] Unknown command: %~1
goto :help

:dev
echo [INFO] Building in development mode...
call :check_deps
call :build_frontend
call :build_backend debug
echo [SUCCESS] Development build complete!
echo [INFO] Run 'build.bat run' to start the server
goto :eof

:release
echo [INFO] Building in release mode...
call :check_deps
call :build_frontend
call :build_backend release
echo [SUCCESS] Release build complete!
echo [INFO] Binary located at: target\release\api.exe
goto :eof

:frontend
call :check_deps
call :build_frontend
goto :eof

:backend
call :check_deps
call :build_backend debug
goto :eof

:clean
echo [INFO] Cleaning build artifacts...
cd /d "%PROJECT_ROOT%"
cargo clean
if exist "%WEB_DIR%\dist" rmdir /s /q "%WEB_DIR%\dist"
if exist "%WEB_DIR%\node_modules" rmdir /s /q "%WEB_DIR%\node_modules"
if exist "%STATIC_DIR%" rmdir /s /q "%STATIC_DIR%"
echo [SUCCESS] Clean complete
goto :eof

:run
echo [INFO] Starting server...
cd /d "%PROJECT_ROOT%"
cargo run -p api
goto :eof

:help
echo Usage: build.bat [command]
echo.
echo Commands:
echo   dev       - Development build (fast, with debug info)
echo   release   - Release build (optimized, for deployment)
echo   frontend  - Build frontend only
echo   backend   - Build backend only
echo   clean     - Clean all build artifacts
echo   run       - Build and run server
echo   help      - Show this help message
echo.
goto :eof

:check_deps
echo [INFO] Checking dependencies...
node --version >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Node.js not found, please install Node.js first
    exit /b 1
)
cargo --version >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Rust/Cargo not found, please install Rust first
    exit /b 1
)
echo [SUCCESS] Dependencies check passed
exit /b 0

:build_frontend
echo [INFO] Building frontend...
cd /d "%WEB_DIR%"
if not exist "node_modules" (
    echo [INFO] Installing frontend dependencies...
    npm install
)
npm run build
if errorlevel 1 (
    echo [ERROR] Frontend build failed
    exit /b 1
)
echo [SUCCESS] Frontend build complete
exit /b 0

:build_backend
echo [INFO] Building backend (%~1 mode)...
cd /d "%PROJECT_ROOT%"
if "%~1"=="release" (
    cargo build --release -p api
) else (
    cargo build -p api
)
if errorlevel 1 (
    echo [ERROR] Backend build failed
    exit /b 1
)
echo [SUCCESS] Backend build complete
exit /b 0

:eof
