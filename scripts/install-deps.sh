#!/usr/bin/env bash
# ESP32-C3 开发环境 — 优先 Homebrew 预编译包，避免 cargo install 编译 OpenSSL（极慢）
set -euo pipefail

export HOMEBREW_NO_AUTO_UPDATE=1
export HOMEBREW_NO_INSTALL_CLEANUP=1

ARCH="$(uname -m)"
case "$ARCH" in
  arm64) TRIPLE="aarch64-apple-darwin" ;;
  x86_64) TRIPLE="x86_64-apple-darwin" ;;
  *) echo "Unsupported arch: $ARCH"; exit 1 ;;
esac

BREW_BIN="$(brew --prefix)/bin"
mkdir -p "$BREW_BIN"

echo "==> Homebrew 安装 CLI 与构建工具..."
brew bundle --file="$(dirname "$0")/../Brewfile"

echo "==> espup（无 Homebrew formula，下载 GitHub 预编译二进制）..."
curl -fsSL "https://github.com/esp-rs/espup/releases/latest/download/espup-${TRIPLE}" \
  -o "$BREW_BIN/espup"
chmod +x "$BREW_BIN/espup"

echo "==> ldproxy（无 Homebrew formula，体积小，一次性编译 ~2 分钟）..."
if ! command -v ldproxy >/dev/null 2>&1; then
  cargo install ldproxy --locked
fi

echo "==> ESP Rust 工具链（RISC-V + esp-idf std）..."
espup install --std --targets esp32c3

echo ""
echo "完成。进入项目目录后加载环境（三选一）:"
echo "  1. direnv（推荐，自动）: brew install direnv && direnv allow"
echo "  2. 手动: source scripts/env.sh"
echo "  3. Cursor/VS Code: 打开本项目，集成终端已配置 PATH"
echo ""
echo "  cd $(dirname "$0")/.."
echo "  ./cargo build --example 01_blink"
echo "  ./cargo run --example 01_blink"
