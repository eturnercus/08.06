@echo off
setlocal EnableExtensions EnableDelayedExpansion

REM =============================================================================
REM NeuroForge — автономная production-сборка для Windows
REM Создатель и правообладатель: eturnercus
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
echo [FAIL] Неизвестный аргумент: %~1
goto show_help
:args_done

echo.
echo ============================================================
echo   NeuroForge — сборка для Windows
echo   Создатель: eturnercus ^| Copyright (c) 2026
echo ============================================================
echo.

REM --- Node.js ---
where node >nul 2>&1
if errorlevel 1 (
    echo [FAIL] Node.js не найден. Установите Node.js 22+: https://nodejs.org/
    goto error_exit
)
where npm >nul 2>&1
if errorlevel 1 (
    echo [FAIL] npm не найден.
    goto error_exit
)
for /f "tokens=1 delims=v" %%a in ('node -v') do set NODE_RAW=%%a
for /f "tokens=1 delims=." %%a in ("!NODE_RAW!") do set NODE_MAJOR=%%a
if !NODE_MAJOR! LSS 22 (
    echo [FAIL] Требуется Node.js 22+, найден: v!NODE_RAW!
    goto error_exit
)
echo [ OK ] Node.js v!NODE_RAW!

REM --- Rust ---
where rustc >nul 2>&1
if errorlevel 1 (
    echo [INFO] Rust не найден.
    echo        Установите: https://rustup.rs/
    echo        После установки перезапустите терминал и снова запустите этот скрипт.
    goto error_exit
)
where cargo >nul 2>&1
if errorlevel 1 (
    echo [FAIL] cargo не найден.
    goto error_exit
)
for /f "tokens=2" %%v in ('rustc --version') do set RUST_VER=%%v
echo [ OK ] Rust !RUST_VER!

REM --- MSVC (рекомендуется для Tauri на Windows) ---
where cl >nul 2>&1
if errorlevel 1 (
    echo [WARN] MSVC ^(cl.exe^) не найден в PATH.
    echo        Установите "Desktop development with C++" из Visual Studio Build Tools:
    echo        https://visualstudio.microsoft.com/visual-cpp-build-tools/
    echo        Или запустите этот .bat из "x64 Native Tools Command Prompt".
)

REM --- WebView2 ---
reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" >nul 2>&1
if errorlevel 1 (
    reg query "HKLM\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" >nul 2>&1
    if errorlevel 1 (
        echo [WARN] WebView2 Runtime может быть не установлен.
        echo        Скачайте: https://developer.microsoft.com/microsoft-edge/webview2/
    )
)

echo [INFO] Установка npm-зависимостей...
if exist package-lock.json (
    call npm ci
) else (
    call npm install
)
if errorlevel 1 goto error_exit
echo [ OK ] npm-зависимости установлены

if "%SKIP_LINT%"=="0" (
    echo [INFO] Проверка TypeScript ^(lint^)...
    call npm run lint
    if errorlevel 1 goto error_exit
    echo [ OK ] Lint пройден
)

if "%CLEAN%"=="1" (
    echo [INFO] Очистка cargo target...
    pushd src-tauri
    call cargo clean
    popd
)

echo [INFO] Production-сборка Tauri ^(это может занять несколько минут^)...
call npm run tauri build
if errorlevel 1 goto error_exit

set "BINARY=%ROOT%\src-tauri\target\release\neuroforge.exe"
set "BUNDLE=%ROOT%\src-tauri\target\release\bundle"

echo.
echo ============================================================
echo [ OK ] Сборка завершена успешно!
echo ============================================================
echo.
echo [INFO] Бинарник:
if exist "%BINARY%" (
    echo   %BINARY%
) else (
    echo [WARN] Бинарник не найден: %BINARY%
)
echo.
echo [INFO] Установочные пакеты:
if exist "%BUNDLE%\msi" dir /b "%BUNDLE%\msi\*.msi" 2>nul
if exist "%BUNDLE%\nsis" dir /b "%BUNDLE%\nsis\*.exe" 2>nul
echo.
echo [INFO] Запуск: %BINARY%
echo.
goto success_exit

:show_help
echo.
echo Использование: build-windows.bat [опции]
echo.
echo   --skip-lint   Пропустить npm run lint
echo   --clean       cargo clean перед сборкой
echo   -h, --help    Справка
echo.
echo Требования:
echo   - Node.js 22+
echo   - Rust stable (rustup)
echo   - Visual Studio Build Tools (MSVC)
echo   - WebView2 Runtime
echo.
echo Артефакты: src-tauri\target\release\bundle\
echo.
exit /b 0

:error_exit
echo.
echo [FAIL] Сборка прервана с ошибкой.
pause
exit /b 1

:success_exit
exit /b 0
