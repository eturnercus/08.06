# AGENTS.md

## NeuroForge

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
- Settings persist to `~/.local/share/neuroforge/settings.json`.
- **UI changes require `npm run tauri:build`** — the release binary embeds the Vite `dist/` bundle at compile time; `npm run build` alone is not enough for the release binary.
- **Where to find key features:**
  - **Agent Studio** (10 orchestration strategies, 12 roles, 14 tools, permissions, resources): nav rail → **Агенты / Agents** → tabs Группы / Редактор / Монитор.
  - **79+ settings in 9 categories**: nav → **Настройки / Settings** (RAM, CPU, inference, injections, AI memory, internet, permissions, appearance, advanced).
  - **Per-chat permissions & memory level (CHAT_ONLY / MODEL_SHARED / GLOBAL)**: nav → **Чаты** → bottom panel in chat sidebar.
  - **DuckDuckGo search + activity log**: nav → **Сеть / Network**.
  - **Hugging Face browser**: nav → **Модели / Models** → Hugging Face tab.
  - **Feature checklist**: nav → **Справка / Help**.
- For automated UI screenshots, target window name `NeuroForge` (not the small `neuroforge` child window): `WID=$(xdotool search --name NeuroForge | head -1)`.
- **Token streaming:** enable **Настройки → Вывод → Streaming** or **Инновации → Thought streaming**. Chat emits Tauri event `chat-stream`; agents emit `agent-stream` (live text in **Агенты → Монитор**). Embedded llama streams per decoded token; `llama-cli` streams from process stdout (may batch if the binary buffers).
