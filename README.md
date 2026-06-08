# NeuroForge

**Создатель и правообладатель: [eturnercus](https://github.com/eturnercus)**  
**Copyright © 2026 eturnercus. All Rights Reserved.**

NeuroForge — кроссплатформенное приложение (Windows / Linux) для локального запуска нейросетей с Hugging Face и пользовательских моделей.

> Полный перечень функций и юридическая защита прав: [`COPYRIGHT.md`](COPYRIGHT.md)  
> Условия использования: [`LICENSE`](LICENSE)

## Возможности

### Команды агентов ИИ
- 10 стратегий оркестровки, 12 ролей, 14 инструментов
- Права и ресурсы на агента, настройки группы, монитор в реальном времени
- Agent Studio: **Агенты → Группы / Редактор / Монитор**

### Память ИИ
- STM (5–200 сообщений), LTM с важностью
- Уровни: `CHAT_ONLY`, `MODEL_SHARED`, `GLOBAL`
- Перенос между чатами и моделями

### Настройки и безопасность
- **79+ настроек в 9 категориях** (RAM, CPU, вывод, инструкции, память, интернет, права, UI, расширенные)
- Изоляция сети, DuckDuckGo, монитор HTTP
- Глобальные системные вводные в каждое сообщение
- Камера, микрофон, экран, файлы

### Интерфейс
- Material 3, RU/EN, 9-шаговый онбординг, подсказки
- Hugging Face браузер, несколько чатов с правами

## Быстрая сборка (одной командой)

### Linux

```bash
chmod +x build-linux.sh
./build-linux.sh
```

Опции: `--skip-deps`, `--skip-lint`, `--clean`, `--help`

### Windows

```cmd
build-windows.bat
```

Опции: `--skip-lint`, `--clean`, `--help`

> **Важно для Windows:** файл `src-tauri/icons/icon.ico` должен быть настоящим ICO (не PNG).
> Скрипт генерирует его автоматически (нужен Python + `pip install pillow`).
> Вручную: `python scripts/generate-icons.py`

Скрипты автоматически:
1. Проверяют Node.js 22+, Rust, системные зависимости
2. Устанавливают npm-пакеты (`npm ci` / `npm install`)
3. Запускают lint и `npm run tauri build`
4. Показывают пути к `.exe`, `.deb`, `.AppImage`, `.msi`

### Ручная сборка

```bash
npm install
npm run lint
npm run tauri:build
```

Артефакты: `src-tauri/target/release/bundle/`

## Требования

| Платформа | Зависимости |
|-----------|-------------|
| **Linux** | Node.js 22+, Rust stable, `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `patchelf`, `libssl-dev` |
| **Windows** | Node.js 22+, Rust stable, Visual Studio Build Tools (MSVC), WebView2 Runtime |

## GitHub Actions

Workflow `.github/workflows/build.yml` собирает артефакты для Linux и Windows. Релизы — при push тега `v*`.

## Авторские права

Программное обеспечение, интерфейс, архитектура Agent Studio, система памяти, настройки и документация являются интеллектуальной собственностью **eturnercus**.

Запрещено распространение, коммерческое использование и снятие копирайта без письменного разрешения правообладателя. Подробности — в [`LICENSE`](LICENSE) и [`COPYRIGHT.md`](COPYRIGHT.md).
