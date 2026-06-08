@echo off
setlocal EnableExtensions EnableDelayedExpansion
chcp 65001 >nul 2>&1

REM =============================================================================
REM NeuroForge - Windows production build
REM Creator and copyright holder: eturnercus
REM Copyright (c) 2026 eturnercus. All Rights Reserved.
REM =============================================================================

cd /d "%~dp0"
set "ROOT=%CD%"
set SKIP_LINT=0
set CLEAN=0

:parse_args
if "%~1"=="" goto args_done
if /i "%~1"=="--skip-lint" (set SKIP_LINT=1 & shift & goto parse_args)
if /i "%~1"=="--clean" (set CLEAN=1 & shift & goto parse_args)
if /i "%~1"=="-h" goto show_help
if /i "%~1"=="--help" goto show_help
echo [FAIL] Unknown argument: %~1
goto show_help
:args_done

echo.
echo ============================================================
echo   NeuroForge - Windows build
echo   Creator: eturnercus ^| Copyright (c) 2026
echo ============================================================
echo.

REM --- Node.js ---
where node >nul 2>&1
if errorlevel 1 (
    echo [FAIL] Node.js not found. Install Node.js 22+: https://nodejs.org/
    goto error_exit
)
where npm >nul 2>&1
if errorlevel 1 (
    echo [FAIL] npm not found.
    goto error_exit
)
for /f "tokens=1 delims=v" %%a in ('node -v') do set NODE_RAW=%%a
for /f "tokens=1 delims=." %%a in ("!NODE_RAW!") do set NODE_MAJOR=%%a
if !NODE_MAJOR! LSS 22 (
    echo [FAIL] Node.js 22+ required, found: v!NODE_RAW!
    goto error_exit
)
echo [ OK ] Node.js v!NODE_RAW!

REM --- Rust ---
where rustc >nul 2>&1
if errorlevel 1 (
    echo [FAIL] Rust not found. Install: https://rustup.rs/
    echo        Restart terminal after install, then run this script again.
    goto error_exit
)
where cargo >nul 2>&1
if errorlevel 1 (
    echo [FAIL] cargo not found.
    goto error_exit
)
for /f "tokens=2" %%v in ('rustc --version') do set RUST_VER=%%v
echo [ OK ] Rust !RUST_VER!

REM --- MSVC ---
where cl >nul 2>&1
if errorlevel 1 (
    echo [WARN] MSVC ^(cl.exe^) not in PATH.
    echo        Install "Desktop development with C++" from VS Build Tools:
    echo        https://visualstudio.microsoft.com/visual-cpp-build-tools/
    echo        Or run this .bat from "x64 Native Tools Command Prompt for VS".
)

REM --- WebView2 ---
reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" >nul 2>&1
if errorlevel 1 (
    reg query "HKLM\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" >nul 2>&1
    if errorlevel 1 (
        echo [WARN] WebView2 Runtime may be missing.
        echo        https://developer.microsoft.com/microsoft-edge/webview2/
    )
)

REM --- Windows icon.ico (must be real ICO format, not a renamed PNG) ---
echo [INFO] Preparing icon.ico for Windows resource compiler...
set ICON_DONE=0
where python >nul 2>&1
if not errorlevel 1 (
    python -c "import PIL" >nul 2>&1 || python -m pip install pillow -q
    python "%ROOT%\scripts\generate-icons.py"
    if not errorlevel 1 set ICON_DONE=1
)
if "!ICON_DONE!"=="0" (
    where py >nul 2>&1
    if not errorlevel 1 (
        py -3 -c "import PIL" >nul 2>&1 || py -3 -m pip install pillow -q
        py -3 "%ROOT%\scripts\generate-icons.py"
        if not errorlevel 1 set ICON_DONE=1
    )
)
if "!ICON_DONE!"=="0" (
    if not exist "%ROOT%\src-tauri\icons\icon.ico" goto icon_fail
    powershell -NoProfile -Command "$b=[IO.File]::ReadAllBytes('%ROOT:\=\\%\src-tauri\icons\icon.ico'); exit -not ($b.Length -gt 4 -and $b[0]-eq 0 -and $b[1]-eq 0 -and $b[2]-eq 1 -and $b[3]-eq 0)" >nul 2>&1
    if errorlevel 1 goto icon_fail
    echo [WARN] Python not found - using existing icon.ico. Run: git pull
)
echo [ OK ] icon.ico ready
goto icon_ok
:icon_fail
echo [FAIL] icon.ico is invalid ^(PNG renamed as .ico^) or missing.
echo        Fix: git pull origin main
echo        Or:  pip install pillow ^&^& python scripts\generate-icons.py
goto error_exit
:icon_ok

echo [INFO] Installing npm dependencies...
if exist package-lock.json (
    call npm ci
) else (
    call npm install
)
if errorlevel 1 goto error_exit
echo [ OK ] npm dependencies installed

if "%SKIP_LINT%"=="0" (
    echo [INFO] TypeScript lint...
    call npm run lint
    if errorlevel 1 goto error_exit
    echo [ OK ] Lint passed
)

if "%CLEAN%"=="1" (
    echo [INFO] cargo clean...
    pushd src-tauri
    call cargo clean
    popd
)

echo [INFO] Tauri production build ^(may take several minutes^)...
call npm run tauri build
if errorlevel 1 goto error_exit

set "BINARY=%ROOT%\src-tauri\target\release\neuroforge.exe"
set "BUNDLE=%ROOT%\src-tauri\target\release\bundle"

echo.
echo ============================================================
echo [ OK ] Build completed successfully!
echo ============================================================
echo.
echo [INFO] Binary:
if exist "%BINARY%" (
    echo   %BINARY%
) else (
    echo [WARN] Binary not found: %BINARY%
)
echo.
echo [INFO] Installers:
if exist "%BUNDLE%\msi" dir /b "%BUNDLE%\msi\*.msi" 2>nul
if exist "%BUNDLE%\nsis" dir /b "%BUNDLE%\nsis\*.exe" 2>nul
echo.
echo [INFO] Run: %BINARY%
echo.
goto success_exit

:show_help
echo.
echo Usage: build-windows.bat [options]
echo.
echo   --skip-lint   Skip npm run lint
echo   --clean       Run cargo clean before build
echo   -h, --help    Show help
echo.
echo Requirements:
echo   - Node.js 22+
echo   - Rust stable (rustup)
echo   - Visual Studio Build Tools (MSVC) recommended
echo   - WebView2 Runtime
echo.
echo Output: src-tauri\target\release\bundle\
echo.
exit /b 0

:error_exit
echo.
echo [FAIL] Build failed.
pause
exit /b 1

:success_exit
exit /b 0
