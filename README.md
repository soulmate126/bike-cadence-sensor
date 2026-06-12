# DIY 自行车踏频器 — ESP32-C3 Rust 工程

基于 [esp-rs](https://github.com/esp-rs) 官方 **esp-idf-template**（ESP-IDF + Rust `std`），目标芯片 **ESP32-C3 SuperMini**。

## 为什么之前 `cargo install` 很慢？

| 方式 | 耗时 | 原因 |
|------|------|------|
| `cargo install espup espflash …` | 15–30+ 分钟 | 从源码编译 **OpenSSL / aws-lc-sys / ring** 等原生依赖 |
| **Homebrew bottle** | 秒级～数分钟 | 预编译二进制，直接 pour |
| **GitHub 预编译 espup** | ~10 秒 | 官方 release 二进制 |
| **首次 `cargo build`** | 10–30 分钟 | 下载 ESP-IDF v5.5.3 并编译 C 组件（仅第一次） |

**推荐策略（本项目采用）**：能用 Homebrew 的用 Homebrew；`espup` 下预编译包；仅 `ldproxy` 用 `cargo install`（~2 分钟，无 Homebrew formula）。

### 重要：PATH 顺序

若同时安装了 `brew install rust` 与 rustup，**必须**让 rustup 的 cargo 优先，否则 `build-std` 不生效：

```bash
export PATH="$HOME/.cargo/bin:$(brew --prefix)/bin:$PATH"
```

---

## 第一部分：检查当前环境

```bash
rustc --version    # Rust 编译器版本
cargo --version    # 包管理与构建
brew --version     # macOS 包管理器
python3 --version  # ESP-IDF 构建脚本依赖（建议 3.12）
```

---

## 第二部分：安装依赖（Homebrew 为主）

### 一键安装

```bash
cd bike-cadence-sensor
chmod +x scripts/install-deps.sh
./scripts/install-deps.sh
```

### 或手动逐步安装

```bash
# 1. Homebrew 安装 CLI 与构建链（预编译，快）
export HOMEBREW_NO_AUTO_UPDATE=1
brew bundle --file=Brewfile

# 2. espup — 无 Homebrew formula，下载 GitHub 预编译二进制
ARCH=$(uname -m); [ "$ARCH" = arm64 ] && T=aarch64-apple-darwin || T=x86_64-apple-darwin
curl -fsSL "https://github.com/esp-rs/espup/releases/latest/download/espup-${T}" \
  -o "$(brew --prefix)/bin/espup" && chmod +x "$(brew --prefix)/bin/espup"

# 3. ldproxy — 链接 ESP-IDF 所需（体积小，仅此项 cargo install）
cargo install ldproxy --locked

# 4. ESP Rust 工具链（ESP32-C3 = RISC-V，std 工程）
espup install --std --targets esp32c3
```

### 各工具用途

| 工具 | 用途 | 安装方式 |
|------|------|----------|
| **rust** | Rust 编译器 / cargo | `brew install rust` |
| **cargo-generate** | 从 esp-idf-template 生成工程 | `brew install cargo-generate` |
| **espflash** | 编译后烧录 + 串口 monitor | `brew install espflash` |
| **esptool** | 底层 Flash 工具（espflash 也会用到） | `brew install esptool` |
| **cmake / ninja** | ESP-IDF 构建 | Homebrew |
| **python@3.12** | ESP-IDF 脚本 | Homebrew |
| **espup** | 安装 ESP 专用 Rust target / 工具链 | GitHub 预编译二进制 |
| **ldproxy** | `.cargo/config.toml` 中配置的 linker | `cargo install ldproxy` |

每次新开终端（若 `~/export-esp.sh` 非空）：

```bash
source ~/export-esp.sh
```

---

## 第三部分：创建项目（已完成）

```bash
cargo generate esp-rs/esp-idf-template cargo \
  --name bike-cadence-sensor \
  --define mcu=esp32c3 \
  --define advanced=false
```

`.cargo/config.toml` 已设置 `ESP_IDF_TOOLS_INSTALL_DIR = "global"`，避免每个项目重复下载 ESP-IDF 工具。

---

## 第四部分：编译与烧录

### 识别 ESP32-C3 串口（macOS）

USB 连接 SuperMini 后：

```bash
ls /dev/cu.usb*
# 常见: /dev/cu.usbmodem* 或 /dev/cu.usbserial-*

espflash board-info          # 自动探测芯片
espflash flash --monitor     # 需先编译出 bin
```

### 编译

```bash
cd bike-cadence-sensor

# 首次会下载 ESP-IDF v5.5.3 并编译 C 组件，约 10–30 分钟，属正常
cargo build --example 01_blink
cargo build --example 02_hall_input
cargo build --example 03_ssd1306
cargo build --example 04_ble_advertise
```

### 烧录 + 串口日志

`.cargo/config.toml` 已配置 `runner = "espflash flash --monitor"`，因此：

```bash
cargo run --example 01_blink
# 等价于 build + espflash flash --monitor
```

指定串口（多设备时）：

```bash
ESPFLASH_PORT=/dev/cu.usbmodem1101 cargo run --example 01_blink
```

---

## 第五～八部分：示例说明

| 示例 | 命令 | 说明 |
|------|------|------|
| GPIO 点灯 | `cargo run --example 01_blink` | 板载 LED **GPIO8**，500ms 翻转 |
| 霍尔输入 | `cargo run --example 02_hall_input` | **GPIO3** 上拉，下降沿计数（可用杜邦线短接 GND 模拟） |
| OLED | `cargo run --example 03_ssd1306` | I2C **SDA=5, SCL=6**，显示 `Hello Bike` |
| BLE 广播 | `cargo run --example 04_ble_advertise` | 设备名 **DIY Cadence Sensor** |

引脚定义见 `src/board/mod.rs`，可按实际接线修改。

---

## 第九部分：目录结构（可扩展）

```
bike-cadence-sensor/
├── Brewfile                 # Homebrew 依赖清单
├── scripts/
│   └── install-deps.sh      # 一键环境安装
├── .cargo/config.toml       # target / espflash runner / ESP-IDF 版本
├── sdkconfig.defaults       # BLE / 栈大小等 Kconfig 默认值
├── examples/
│   ├── 01_blink.rs
│   ├── 02_hall_input.rs
│   ├── 03_ssd1306.rs
│   └── 04_ble_advertise.rs
└── src/
    ├── main.rs              # 主固件入口（占位）
    ├── lib.rs
    ├── board/               # 引脚与板级配置
    ├── sensors/             # 霍尔传感器（→ hall.rs）
    ├── cadence/             # RPM / 踏频计算
    ├── ble/                 # BLE CSC 服务（→ csc.rs，对接华为 GT5 Pro）
    ├── display/             # SSD1306 UI
    ├── gps/                 # GPS NMEA（预留）
    └── navigation/          # 路书导航（预留）
```

后续 **CSC（Cycling Speed and Cadence）** 标准服务 UUID `0x1816`，在 `src/ble/csc.rs` 实现 GATT Server，与华为运动健康 / GT5 Pro 配对。

---

## 第十部分：验证步骤

```bash
# 1. 工具链
espup --version && espflash --version && cargo-generate --version && ldproxy 2>&1 | head -1

# 2. 编译（不连板也可）
cargo build --example 01_blink

# 3. 连板烧录
cargo run --example 01_blink
# 预期：串口看到日志，GPIO8 LED 闪烁

# 4. BLE
cargo run --example 04_ble_advertise
# 手机 nRF Connect 可扫到 "DIY Cadence Sensor"
```

---

## 常见问题排查

| 现象 | 可能原因 | 处理 |
|------|----------|------|
| `cargo install` 极慢 / 失败 | 编译 OpenSSL | 改用 **Homebrew** 或 **GitHub 预编译 espup** |
| `brew install` 卡住 | Homebrew auto-update | `export HOMEBREW_NO_AUTO_UPDATE=1` |
| 找不到串口 | 驱动 / 线材 | 换数据线；`ls /dev/cu.*`；安装 CP210x/CH340 驱动 |
| `espflash` 无法连接 | 端口占用 | 关闭其他串口监视器；指定 `ESPFLASH_PORT` |
| 首次 `cargo build` 很久 | 下载并编译 ESP-IDF | 正常；确保网络稳定；`ESP_IDF_TOOLS_INSTALL_DIR=global` |
| `ldproxy` not found | PATH | `export PATH="$HOME/.cargo/bin:$(brew --prefix)/bin:$PATH"` |
| BLE 扫不到 | sdkconfig / 内存 | 检查 `sdkconfig.defaults` 中 `CONFIG_BT_*`；参考 esp-idf-svc `bt_gatt_server` 示例 |
| OLED 无显示 | 接线 / 地址 | 确认 3.3V、SDA/SCL、I2C 地址 0x3C |

---

## Cursor / VS Code

推荐扩展：**rust-analyzer**。Rust target 为 `riscv32imc-esp-espidf`，IDE 可能提示部分 cfg 未知，可忽略或参考 [Rust on ESP Book](https://docs.espressif.com/projects/rust/book/)。

---

## 参考

- [esp-rs esp-idf-template](https://github.com/esp-rs/esp-idf-template)
- [Rust on ESP Book](https://docs.espressif.com/projects/rust/book/)
- [esp-idf-svc BLE GATT 示例](https://github.com/esp-rs/esp-idf-svc/blob/master/examples/bt_gatt_server.rs)
- CSC 规范：Bluetooth SIG Cycling Speed and Cadence Service `0x1816`
