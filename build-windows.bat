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

REM --- MSVC (required for embedded llama-cpp build) ---
set "BUILD_MODE=embedded"
where cl >nul 2>&1
if errorlevel 1 call :setup_msvc
where cl >nul 2>&1
if errorlevel 1 (
    echo [WARN] MSVC ^(cl.exe^) not in PATH.
    echo        Install "Desktop development with C++" from VS Build Tools:
    echo        https://visualstudio.microsoft.com/visual-cpp-build-tools/
    echo        Build will use llama-cli subprocess mode ^(no embedded llama-cpp^).
    set "BUILD_MODE=cli"
) else (
    echo [ OK ] MSVC ^(cl.exe^) available
)

REM --- LLVM / libclang (bindgen for llama-cpp-sys) ---
set "LIBCLANG_PATH="
call :setup_llvm
if defined LIBCLANG_PATH (
    echo [ OK ] LIBCLANG_PATH=!LIBCLANG_PATH!
) else if "!BUILD_MODE!"=="embedded" (
    echo [WARN] libclang not found ^(bindgen needs LLVM^).
    echo        Install: winget install LLVM.LLVM
    echo        Or set LIBCLANG_PATH to the folder containing libclang.dll
    echo        Switching to llama-cli subprocess mode.
    set "BUILD_MODE=cli"
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

if /i "!BUILD_MODE!"=="cli" (
    echo [INFO] Downloading llama-cli for subprocess inference...
    call :download_llama
    if errorlevel 1 goto error_exit
    echo [INFO] Tauri production build ^(llama-cli mode, no embedded llama-cpp^)...
    call npm run tauri -- build -- --no-default-features
) else (
    echo [INFO] Tauri production build ^(embedded llama-cpp, may take several minutes^)...
    call npm run tauri build
    if errorlevel 1 (
        echo [WARN] Embedded build failed. Retrying with llama-cli subprocess mode...
        call :download_llama
        if errorlevel 1 goto error_exit
        call npm run tauri -- build -- --no-default-features
    )
)
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

:setup_msvc
for %%E in (2022 2019) do (
    if exist "C:\Program Files\Microsoft Visual Studio\%%E\Community\VC\Auxiliary\Build\vcvars64.bat" (
        call "C:\Program Files\Microsoft Visual Studio\%%E\Community\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
        exit /b 0
    )
    if exist "C:\Program Files\Microsoft Visual Studio\%%E\BuildTools\VC\Auxiliary\Build\vcvars64.bat" (
        call "C:\Program Files\Microsoft Visual Studio\%%E\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
        exit /b 0
    )
    if exist "C:\Program Files (x86)\Microsoft Visual Studio\%%E\BuildTools\VC\Auxiliary\Build\vcvars64.bat" (
        call "C:\Program Files (x86)\Microsoft Visual Studio\%%E\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
        exit /b 0
    )
)
exit /b 1

:setup_llvm
if exist "C:\Program Files\LLVM\bin\libclang.dll" (
    set "LIBCLANG_PATH=C:\Program Files\LLVM\bin"
    exit /b 0
)
if exist "C:\Program Files (x86)\LLVM\bin\libclang.dll" (
    set "LIBCLANG_PATH=C:\Program Files (x86)\LLVM\bin"
    exit /b 0
)
where libclang.dll >nul 2>&1
if not errorlevel 1 (
    for /f "delims=" %%D in ('where libclang.dll 2^>nul') do (
        set "LIBCLANG_PATH=%%~dpD"
        set "LIBCLANG_PATH=!LIBCLANG_PATH:~0,-1!"
        exit /b 0
    )
)
exit /b 1

:download_llama
where powershell >nul 2>&1
if errorlevel 1 (
    echo [FAIL] PowerShell required to download llama-cli.
    exit /b 1
)
powershell -NoProfile -ExecutionPolicy Bypass -File "%ROOT%\scripts\download-llama-win.ps1"
if errorlevel 1 exit /b 1
if not exist "%ROOT%\src-tauri\bin\llama\llama-cli.exe" (
    echo [FAIL] llama-cli.exe not found after download.
    exit /b 1
)
echo [ OK ] llama-cli.exe ready
exit /b 0

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
echo   - For embedded llama: VS Build Tools (MSVC) + LLVM (libclang)
echo   - Without LLVM/MSVC: auto-downloads llama-cli (subprocess mode)
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
