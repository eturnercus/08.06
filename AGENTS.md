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

- GUI testing requires a display (`DISPLAY`); use xvfb for headless smoke tests.
- First launch shows language picker (ru/en), then onboarding, then main app.
- Settings persist to `~/.local/share/neuroforge/settings.json`.
