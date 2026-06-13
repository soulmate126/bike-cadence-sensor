#!/usr/bin/env bash
# 在主机上运行 cadence-core 单元测试（绕过 ESP 默认 target）
set -euo pipefail
cd "$(dirname "$0")/.."
HOST=$(rustc -vV | awk '/^host: / { print $2 }')
cargo test -p cadence-core --target "$HOST" "$@"
