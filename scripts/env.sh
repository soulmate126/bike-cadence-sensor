#!/usr/bin/env bash
# 项目环境变量 — 进入目录后自动加载（direnv）或手动 source
#
#   source scripts/env.sh
#
# 作用：让 rustup 的 cargo 优先于 Homebrew rust（build-std 必需）

_env_script_dir="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
export BIKE_CADENCE_SENSOR_ROOT="$(cd "$_env_script_dir/.." && pwd)"

if command -v brew >/dev/null 2>&1; then
  export PATH="$HOME/.cargo/bin:$(brew --prefix)/bin:$PATH"
else
  export PATH="$HOME/.cargo/bin:$PATH"
fi

# espup 生成的环境（文件为空则跳过）
if [[ -s "$HOME/export-esp.sh" ]]; then
  # shellcheck source=/dev/null
  source "$HOME/export-esp.sh"
fi
