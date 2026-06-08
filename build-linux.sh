#!/usr/bin/env bash
# =============================================================================
# NeuroForge — автономная production-сборка для Linux
# Создатель и правообладатель: eturnercus
# Copyright (c) 2026 eturnercus. All Rights Reserved.
# =============================================================================
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${CYAN}[INFO]${NC} $*"; }
ok()    { echo -e "${GREEN}[ OK ]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
fail()  { echo -e "${RED}[FAIL]${NC} $*"; exit 1; }

SKIP_DEPS=0
SKIP_LINT=0
CLEAN=0

usage() {
  cat <<'EOF'
Использование: ./build-linux.sh [опции]

  --skip-deps   Не устанавливать системные пакеты (apt)
  --skip-lint   Пропустить npm run lint
  --clean       cargo clean перед сборкой
  -h, --help    Справка

Требования:
  - Node.js 22+
  - Rust stable (1.85+)
  - Linux: webkit2gtk, appindicator, librsvg, patchelf, libssl

Артефакты: src-tauri/target/release/bundle/
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-deps) SKIP_DEPS=1 ;;
    --skip-lint) SKIP_LINT=1 ;;
    --clean) CLEAN=1 ;;
    -h|--help) usage; exit 0 ;;
    *) fail "Неизвестный аргумент: $1 (используйте --help)" ;;
  esac
  shift
done

echo ""
echo "============================================================"
echo "  NeuroForge — сборка для Linux"
echo "  Создатель: eturnercus | Copyright (c) 2026"
echo "============================================================"
echo ""

# --- Системные зависимости (Debian/Ubuntu) ---
install_linux_deps() {
  if [[ "$SKIP_DEPS" -eq 1 ]]; then
    warn "Пропуск установки системных пакетов (--skip-deps)"
    return
  fi
  if command -v apt-get >/dev/null 2>&1; then
    info "Проверка системных пакетов (apt)..."
    local pkgs=(
      build-essential curl wget pkg-config libssl-dev
      libwebkit2gtk-4.1-dev libayatana-appindicator3-dev
      librsvg2-dev patchelf
    )
    local missing=()
    for p in "${pkgs[@]}"; do
      dpkg -s "$p" >/dev/null 2>&1 || missing+=("$p")
    done
    if [[ ${#missing[@]} -gt 0 ]]; then
      info "Установка: ${missing[*]}"
      sudo DEBIAN_FRONTEND=noninteractive apt-get update -qq
      sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq "${missing[@]}"
    fi
    ok "Системные зависимости готовы"
  elif command -v dnf >/dev/null 2>&1; then
    warn "Fedora/RHEL: установите вручную webkit2gtk4.1-devel, openssl-devel, librsvg2-devel"
  elif command -v pacman >/dev/null 2>&1; then
    warn "Arch: установите webkit2gtk-4.1, libappindicator-gtk3, librsvg, patchelf, openssl"
  else
    warn "Неизвестный дистрибутив — установите Tauri Linux deps вручную"
  fi
}

# --- Node.js ---
ensure_node() {
  if ! command -v node >/dev/null 2>&1; then
    fail "Node.js не найден. Установите Node.js 22+: https://nodejs.org/"
  fi
  local ver major
  ver="$(node -v | sed 's/^v//')"
  major="${ver%%.*}"
  if [[ "$major" -lt 22 ]]; then
    fail "Требуется Node.js 22+, найден: v$ver"
  fi
  command -v npm >/dev/null 2>&1 || fail "npm не найден"
  ok "Node.js v$ver, npm $(npm -v)"
}

# --- Rust ---
ensure_rust() {
  if ! command -v rustc >/dev/null 2>&1; then
    info "Rust не найден — установка через rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
  fi
  if ! command -v cargo >/dev/null 2>&1; then
    fail "cargo не найден после установки Rust"
  fi
  local rust_ver
  rust_ver="$(rustc --version | awk '{print $2}')"
  ok "Rust $rust_ver"
}

# --- Сборка ---
install_linux_deps
ensure_node
ensure_rust

info "Проверка icon.ico (для кросс-сборки Windows)..."
if command -v python3 >/dev/null 2>&1; then
  if ! python3 -c "import PIL" 2>/dev/null; then
    python3 -m pip install pillow -q 2>/dev/null || true
  fi
  python3 "$ROOT/scripts/generate-icons.py" 2>/dev/null || true
fi

info "Установка npm-зависимостей..."
if [[ -f package-lock.json ]]; then
  npm ci
else
  npm install
fi
ok "npm-зависимости установлены"

if [[ "$SKIP_LINT" -eq 0 ]]; then
  info "Проверка TypeScript (lint)..."
  npm run lint
  ok "Lint пройден"
fi

if [[ "$CLEAN" -eq 1 ]]; then
  info "Очистка cargo target..."
  (cd src-tauri && cargo clean)
fi

info "Production-сборка Tauri (это может занять несколько минут)..."
export CI="${CI:-false}"
npm run tauri build

BUNDLE="$ROOT/src-tauri/target/release/bundle"
BINARY="$ROOT/src-tauri/target/release/neuroforge"

echo ""
echo "============================================================"
ok "Сборка завершена успешно!"
echo "============================================================"
echo ""
info "Бинарник:"
[[ -f "$BINARY" ]] && echo "  $BINARY" || warn "Бинарник не найден: $BINARY"
echo ""
info "Установочные пакеты:"
if [[ -d "$BUNDLE" ]]; then
  find "$BUNDLE" -maxdepth 3 \( -name '*.deb' -o -name '*.AppImage' -o -name '*.rpm' \) -print 2>/dev/null | while read -r f; do
    echo "  $f"
  done
  if ! find "$BUNDLE" -maxdepth 3 \( -name '*.deb' -o -name '*.AppImage' -o -name '*.rpm' \) -print -quit 2>/dev/null | grep -q .; then
    warn "Пакеты .deb/.AppImage/.rpm не найдены — проверьте лог выше"
  fi
else
  warn "Каталог bundle не найден: $BUNDLE"
fi
echo ""
info "Запуск: $BINARY"
echo ""
