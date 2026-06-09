# Silenium

**Создатель и правообладатель: [eturnercus](https://github.com/eturnercus)**  
**Copyright © 2026 eturnercus. All Rights Reserved.**

**Silenium** — кроссплатформенное настольное приложение (Windows / Linux) для **локального** запуска нейросетей, команд агентов и автоматизации с полным контролем над данными, сетью и устройствами.

> Название от *silo* (изолированное хранилище) + суффикс *-enium*: ИИ работает **у вас**, в «силосе», а не в облаке.  
> Не путать с **[Selenium](https://www.selenium.dev/)** — это другой продукт (фреймворк для автотестов браузера, другое написание).

Юридическая информация: [`COPYRIGHT.md`](COPYRIGHT.md) · [`LICENSE`](LICENSE)

---

## Зачем Silenium

| Проблема | Решение в Silenium |
|----------|-------------------|
| Данные уходят в облако | Инференс GGUF/llama.cpp на вашем ПК; настройки и память на диске |
| Нейросеть «сама лезет в интернет» | Три режима сети + per-chat права + журнал каждого HTTP-запроса |
| Один чат — одна модель | Несколько чатов, Agent Studio с **10 стратегиями** и разными моделями на агента |
| Нет долгой памяти | **STM** (контекст диалога) и **LTM** (факты между чатами) |
| Агент двигает вашу мышь | **Две мыши**: системная — ваша; фиолетовый курсор — только ИИ в агент-браузере |
| Сотни настроек «для галочки» | Движок настроек реально влияет на инференс, память, сеть и агентов |

---

## Что уже реализовано

### Чаты и инференс
- Несколько чатов с отдельными моделями, температурой, лимитами RAM/токенов, системным промптом
- **Потоковая генерация** токенов (`chat-stream`), кнопка **Стоп**, блоки «мышления», метаданные сообщений
- Встроенный **GGUF** (llama.cpp) + subprocess `llama-cli`; загрузка с **Hugging Face**
- Экспорт / удаление чатов, автоскролл, тёмная/светлая тема

### Agent Studio
- **10** стратегий оркестровки (sequential, parallel, voting, map_reduce, …)
- **12** ролей, **17** инструментов (включая `browser_navigate`, `browser_search`, `browser_click`)
- Права на агента (интернет, экран, STM/LTM, …), ресурсы, триггеры, монитор в реальном времени (`agent-stream`)
- Привязка **команды агентов** к чату

### Память
- STM / LTM, уровни `CHAT_ONLY` · `MODEL_SHARED` · `GLOBAL`
- Перенос записей между чатами, консолидация STM → LTM
- Опциональное шифрование памяти на диске

### Сеть и безопасность
- Режимы изоляции, белый список API, блокировка приватных IP
- DuckDuckGo-поиск с логированием, `agent_fetch`, вкладка **Журнал аудита**
- Prompt-injection shield, audit log, data exfiltration guard (настраиваемо)

### Устройства и автоматизация
- Статус камеры / микрофона / экрана / виртуального дисплея
- **Агент-браузер**: встроенный просмотр HTML, навигация и поиск для агентов
- **Dual mouse**: виртуальный курсор ИИ поверх превью; системная мышь не затрагивается
- Включение: *Настройки → Права → Browser automation + Desktop control*

### Настройки (150+ параметров)
- Железо: RAM, CPU affinity, GPU layers, mmap, OOM-политика
- Инференс: streaming, context, flash attention, backends
- **Инновации**: thought streaming, swarm, neuroplastic memory, …
- Безопасность, производительность, глобальные вводные в сообщения, UI

### Интерфейс
- Material 3, **RU/EN**, 9-шаговый онбординг, подсказки
- Боковая панель чата с правами и системными инструкциями (тёмные скроллбары)

---

## Дорожная карта (планируется)

| Направление | Статус |
|-------------|--------|
| Полноценный Chrome/WebView с DOM-кликами (как в IDE-агентах) | Планируется |
| Нативный захват экрана / OCR / STT (сейчас — заготовки API) | Планируется |
| macOS-сборка | Планируется |
| Плагины инструментов агентов (WASM / скрипты) | Исследование |
| Синхронизация LTM между устройствами (E2E) | Исследование |
| ONNX / TensorRT backends (кроме GGUF) | Частично в настройках |

Текущий агент-браузер загружает HTML через сеть и показывает в sandboxed `srcdoc` — этого достаточно для поиска и простых страниц; сложный JS-сайты могут отображаться упрощённо.

---

## Быстрая сборка

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

> **Windows:** `src-tauri/icons/icon.ico` должен быть настоящим ICO. Скрипт генерирует его через Python + Pillow, либо: `python scripts/generate-icons.py`

Скрипты проверяют Node.js 22+, Rust, зависимости, запускают `npm run lint` и `npm run tauri build`.

### Ручная сборка

```bash
npm install
npm run lint
npm run tauri:build
```

Артефакты: `src-tauri/target/release/bundle/`  
Модели по умолчанию: `~/.local/share/silenium/models/` (Linux)

При обновлении с **NeuroForge** папка `~/.local/share/neuroforge` автоматически переименовывается в `silenium`; ключи `localStorage` мигрируют при первом запуске.

---

## Разработка

```bash
npm install
npm run tauri:dev    # Vite :1420 + Tauri
npm run lint
```

| Платформа | Зависимости |
|-----------|-------------|
| **Linux** | Node.js 22+, Rust stable, `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `patchelf`, `libssl-dev` |
| **Windows** | Node.js 22+, Rust stable, MSVC Build Tools, WebView2 |

### Релизы (GitHub)

Сборка CI по умолчанию **не** публикует релиз — только проверяет, что проект собирается.

| Способ | Когда использовать |
|--------|-------------------|
| **Тег `v*`** | `git tag v1.0.1 && git push origin v1.0.1` — после мержа в `main`, версия в `src-tauri/tauri.conf.json` должна совпадать (`1.0.1` → тег `v1.0.1`) |
| **Actions → Build Silenium → Run workflow** | Поле `release_tag`: `v1.0.1` на ветке `main` — соберёт `.deb` + `.msi` и создаст GitHub Release |

Артефакты: `Silenium_*_amd64.deb` (Linux), `Silenium_*_x64_en-US.msi` (Windows).

> Старый релиз **NeuroForge v1.0.0** — до переименования. Первый релиз **Silenium** — с **v1.0.1**.

CI: `.github/workflows/build.yml` — Linux `.deb` + Windows `.msi` (без AppImage, чтобы сборка не падала на `linuxdeploy`).

---

## Стек

- **Frontend:** React 18, TypeScript, Vite, Zustand, i18next, Material 3 CSS
- **Backend:** Rust, Tauri 2, llama.cpp (embedded), reqwest, parking_lot

---

## Авторские права

Программный код, UI, архитектура Agent Studio, подсистема памяти, движок настроек и документация — **интеллектуальная собственность eturnercus**.

Распространение, коммерческое использование и снятие копирайта без письменного разрешения запрещены. Подробности — в [`LICENSE`](LICENSE).
