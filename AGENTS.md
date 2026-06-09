# AGENTS.md

## Silenium

Cross-platform desktop app (Tauri 2 + React + Rust) for local AI inference.

## Cursor Cloud specific instructions

### Services

| Service | Required | Command |
|---------|----------|---------|
| Vite dev server | Dev only | Started automatically by `npm run tauri:dev` on port 1420 |
| Tauri app | Manual test | `npm run tauri:dev` or run built binary |

### System dependencies (Linux)

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf libssl-dev pkg-config
```

Rust **1.85+** required (use `rustup update stable`).

### Commands

```bash
npm install
npm run lint
npm run build
npm run tauri:build    # production binary + .deb/.AppImage
npm run tauri:dev      # development with hot reload
```

Built artifacts: `src-tauri/target/release/bundle/`

### Notes

- GUI testing requires a display (`DISPLAY=:1` in Cloud VMs).
- First launch: language picker → 9-step onboarding → main app.
- Settings persist to `~/.local/share/silenium/settings.json` (legacy `neuroforge` folder is renamed on first run).
- **UI changes require `npm run tauri:build`** — the release binary embeds the Vite `dist/` bundle at compile time; `npm run build` alone is not enough for the release binary.
- **Where to find key features:**
  - **Agent Studio** (10 orchestration strategies, 12 roles, 14 tools, permissions, resources): nav rail → **Агенты / Agents** → tabs Группы / Редактор / Монитор.
  - **150+ settings**: nav → **Настройки / Settings** (RAM, CPU, inference, innovations, security, performance, injections, AI memory, internet, permissions, appearance, advanced).
  - **Agent browser + dual AI mouse**: nav → **Устройства / Devices** (requires Browser automation + Desktop control in Settings → Permissions).
  - **Live WebView DOM mode**: Devices → Agent browser → **DOM WebView** toggle opens a separate Tauri webview window; agent clicks use `elementFromPoint` / CSS `querySelector` via injected JS bridge. Preview iframe uses postMessage for link clicks.
  - **Screen capture / OCR / STT**: Devices → capture buttons. Linux capture: `scrot`, `grim`, `gnome-screenshot`, `import`, or `ffmpeg x11grab`. OCR needs `tesseract` (+ `tesseract-ocr-rus`). STT needs `whisper` or `whisper-cli` with a model. Enable screen/mic in Settings → Permissions → Devices first.
  - **Per-chat permissions & memory level (CHAT_ONLY / MODEL_SHARED / GLOBAL)**: nav → **Чаты** → bottom panel in chat sidebar.
  - **DuckDuckGo search + activity log**: nav → **Сеть / Network**.
  - **Hugging Face browser**: nav → **Модели / Models** → Hugging Face tab.
  - **Feature checklist**: nav → **Справка / Help**.
- For automated UI screenshots, target window name `Silenium` (not the small `silenium` child window): `WID=$(xdotool search --name Silenium | head -1)`.
- **Token streaming:** enable **Настройки → Вывод → Streaming** or **Инновации → Thought streaming**. Chat emits Tauri event `chat-stream`; agents emit `agent-stream` (live text in **Агенты → Монитор**). Embedded llama streams per decoded token; `llama-cli` streams from process stdout (may batch if the binary buffers).
- **Stop generation:** Stop button in chat → `stop_chat`; cancels embedded token loop and kills `llama-cli`.
- **Agent in chat:** assign **Команда агентов** in chat sidebar — composer uses `run_agent_team` instead of single-model `send_chat`.
