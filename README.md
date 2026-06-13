# DIY Cadence Sensor

ESP32-C3 SuperMini + KY-003 霍尔 + SSD1306 OLED + BLE CSC（NimBLE）自制踏频器。  
设备名 **DIY Cadence Sensor**，标准 CSC 服务 `0x1816` / 特征 `0x2A5B`，已验证 nRF Connect 订阅与华为 GT5 Pro 踏频显示。

## Milestones

- [x] 项目骨架（`src/` 模块 + examples）
- [x] GPIO 点灯（`01_blink`）
- [x] GPIO / 霍尔输入（`02_hall_input`）
- [x] RPM 算法（`src/cadence/`，含单元测试）
- [x] BLE CSC 数据结构（`src/ble/csc.rs`）
- [x] BLE GATT Server（`src/ble/server.rs`，NimBLE）
- [x] OLED 本地显示（`03_ssd1306` / `06_hall_oled`）
- [x] nRF Connect 连接 + CSC Notify
- [x] 华为 GT5 Pro 踏频识别
- [x] 主固件三合一（霍尔 + OLED + BLE，`./cargo run`）

协议说明见 [docs/csc-protocol.md](docs/csc-protocol.md)。  
**新电脑 / 换设备**见 [docs/new-machine-setup.md](docs/new-machine-setup.md)。

---

## 快速开始

```bash
git clone git@github.com:soulmate126/bike-cadence-sensor.git
cd bike-cadence-sensor
./scripts/install-deps.sh
./cargo build                              # 无板可编译
./cargo run                                # 有板：烧录主固件（霍尔+OLED+BLE）
```

按模块逐步验证：

```bash
./cargo run --example 01_blink             # LED
./cargo run --example 02_hall_input        # 霍尔接线
./cargo run --example 03_ssd1306         # OLED
./cargo run --example 04_ble_advertise     # 仅广播
./cargo run --example 05_ble_csc           # 霍尔 + BLE
./cargo run --example 06_hall_oled         # 霍尔 + OLED
```

详细步骤、验证顺序与排错见 [docs/new-machine-setup.md](docs/new-machine-setup.md)。

---

## 接线（ESP32-C3 SuperMini）

| 设备 | 引脚 | 说明 |
|------|------|------|
| KY-003 S | **GPIO4** | 磁铁靠近时下降沿计 1 转 |
| KY-003 + / - | 3V3 / GND | 模块板载上拉，ESP 侧 `Pull::Floating` |
| SSD1306 SDA / SCL | **GPIO5 / GPIO6** | I2C 100 kHz，地址 0x3C 或 0x3D |
| SSD1306 VCC / GND | 3V3 / GND | |
| 板载 LED | **GPIO8** | 低电平点亮 |

引脚常量见 `src/board/mod.rs`，可按实际接线修改。

---

## 架构

业务逻辑全部在 `src/`；`examples/` 与 `main.rs` 只做 `init()` + 一行 `run()`，便于分步烧录验证。

```
KY-003 (GPIO4)
    └─ sensors/hall.rs ── cadence/（RPM、去抖、CSC 时间）
            ├─ display/oled.rs   → SSD1306 显示 RPM / COUNT
            └─ ble/server.rs     → NimBLE CSC 0x1816 Notify 0x2A5B
```

| 模块 | 职责 |
|------|------|
| `cadence/` | 踏频算法（200 ms 去抖、3 s 归零、u16 循环累加） |
| `sensors/hall.rs` | 霍尔驱动 + 算法集成 |
| `sensors/hall_debug.rs` | 原始边沿打印（示例 2） |
| `display/oled.rs` | `CadenceOled` UI、`run_hello_demo()` |
| `hardware/i2c.rs` | I2C 探测 / 扫描 / OLED 地址解析 |
| `ble/server.rs` + `csc.rs` | NimBLE GATT CSC 服务 |
| `ble/advertise.rs` | 仅广播（示例 4） |
| `board/` | 引脚定义、`led::run_blink()` |
| `board/config.rs` | 去抖 / 超时 / LED 周期等可调参数 |
| `board/status_led.rs` | GPIO8 连接/踏频状态灯 |
| `util/time.rs` | 统一 `now_ms()` |
| `app/` | `run_full` / `run_hall_ble` / `run_hall_oled` |
| `crates/cadence-core/` | 踏频纯逻辑（主机单元测试） |

主固件：**BLE Bonding**、订阅后立即 notify、OLED 失败降级、GPIO8 状态 LED、霍尔 GPIO 中断。

升级固件后请在手表上**删除并重新添加**踏频器以完成绑定。

主机单元测试：`./scripts/test-cadence.sh`

---

## 示例与主固件

| 目标 | 命令 | 实现 |
|------|------|------|
| **主固件** | `./cargo run` | `app::run_full()` — 霍尔 + OLED + BLE |
| GPIO 点灯 | `./cargo run --example 01_blink` | `board::led::run_blink()` |
| 霍尔接线 | `./cargo run --example 02_hall_input` | `sensors::hall_debug::run_raw_monitor()` |
| OLED Hello | `./cargo run --example 03_ssd1306` | `display::run_hello_demo()` |
| BLE 广播 | `./cargo run --example 04_ble_advertise` | `ble::advertise::run_advertise()` |
| 霍尔 + BLE | `./cargo run --example 05_ble_csc` | `app::run_hall_ble()` |
| 霍尔 + OLED | `./cargo run --example 06_hall_oled` | `app::run_hall_oled()` |

指定串口（多设备时）：

```bash
ESPFLASH_PORT=/dev/cu.usbmodem2201 ./cargo run
```

---

## 目录结构

```
bike-cadence-sensor/
├── Brewfile
├── scripts/
│   ├── install-deps.sh
│   └── env.sh
├── .cargo/config.toml       # target / espflash runner / ESP-IDF 版本
├── sdkconfig.defaults       # NimBLE、栈大小等 Kconfig 默认值
├── examples/                # 薄封装：init() + 一行 run()
│   ├── 01_blink.rs … 06_hall_oled.rs
└── src/
    ├── main.rs              # init() → app::run_full()
    ├── lib.rs
    ├── app/                 # 主循环（full / hall_ble / hall_oled）
    ├── board/               # 引脚、LED
    ├── sensors/             # hall、hall_debug
    ├── cadence/             # RPM / 踏频计算
    ├── display/             # SSD1306
    ├── hardware/            # I2C 工具
    ├── ble/                 # CSC GATT（NimBLE / esp32-nimble）
    ├── util/                # 时间等共用工具
    ├── gps/                 # 预留
    └── navigation/          # 预留
```

---

## BLE 说明

- 栈：**NimBLE**（`esp32-nimble`）。本板 ESP32-C3 上 Bluedroid 广播会返回 `Cmd Disallowed (0x0C)`，NimBLE 正常。
- 服务 UUID：`0x1816`（Cycling Speed and Cadence）
- 特征 UUID：`0x2A5B`（CSC Measurement，Notify）
- 载荷：累计曲柄转数（u16）+ 最后事件时间（1/1024 s，u16），接收端自行算 RPM。

验证：

1. nRF Connect 扫描 **DIY Cadence Sensor** → 连接 → 订阅 `0x2A5B`
2. 转曲柄或靠近磁铁，观察 Notify 递增
3. 华为 GT5 Pro：设置 → 健康与健身设备 → 踏频器

---

基于 [esp-rs](https://github.com/esp-rs) 官方 **esp-idf-template**（ESP-IDF + Rust `std`），目标芯片 **ESP32-C3 SuperMini**。

## 为什么之前 `cargo install` 很慢？

| 方式 | 耗时 | 原因 |
|------|------|------|
| `cargo install espup espflash …` | 15–30+ 分钟 | 从源码编译 **OpenSSL / aws-lc-sys / ring** 等原生依赖 |
| **Homebrew bottle** | 秒级～数分钟 | 预编译二进制，直接 pour |
| **GitHub 预编译 espup** | ~10 秒 | 官方 release 二进制 |
| **首次 `cargo build`** | 10–30 分钟 | 下载 ESP-IDF v5.5.3 并编译 C 组件（仅第一次） |

**推荐策略（本项目采用）**：能用 Homebrew 的用 Homebrew；`espup` 下预编译包；仅 `ldproxy` 用 `cargo install`（~2 分钟，无 Homebrew formula）。

### 重要：PATH 顺序（已写入项目）

若同时安装了 `brew install rust` 与 rustup，**必须**让 rustup 的 cargo 优先，否则 `build-std` 不生效。

| 方式 | 命令 / 操作 |
|------|-------------|
| **项目 cargo 脚本** | `./cargo build`（无需 direnv，推荐） |
| **direnv** | `brew install direnv`，配置 shell hook 后 `direnv allow` |
| **手动 source** | `source scripts/env.sh` |
| **Cursor / VS Code** | 打开本项目，集成终端自动应用 `.vscode/settings.json` |

核心逻辑在 `scripts/env.sh`：

```bash
export PATH="$HOME/.cargo/bin:$(brew --prefix)/bin:$PATH"
```

---

## 环境安装

### 一键安装

```bash
cd bike-cadence-sensor
chmod +x scripts/install-deps.sh
./scripts/install-deps.sh
```

### 或手动逐步安装

```bash
export HOMEBREW_NO_AUTO_UPDATE=1
brew bundle --file=Brewfile

ARCH=$(uname -m); [ "$ARCH" = arm64 ] && T=aarch64-apple-darwin || T=x86_64-apple-darwin
curl -fsSL "https://github.com/esp-rs/espup/releases/latest/download/espup-${T}" \
  -o "$(brew --prefix)/bin/espup" && chmod +x "$(brew --prefix)/bin/espup"

cargo install ldproxy --locked
espup install --std --targets esp32c3
```

每次新开终端（若 `~/export-esp.sh` 非空）：

```bash
source ~/export-esp.sh
```

### 各工具用途

| 工具 | 用途 | 安装方式 |
|------|------|----------|
| **rust** | Rust 编译器 / cargo | `brew install rust` |
| **cargo-generate** | 从 esp-idf-template 生成工程 | `brew install cargo-generate` |
| **espflash** | 编译后烧录 + 串口 monitor | `brew install espflash` |
| **esptool** | 底层 Flash 工具 | `brew install esptool` |
| **cmake / ninja** | ESP-IDF 构建 | Homebrew |
| **python@3.12** | ESP-IDF 脚本 | Homebrew |
| **espup** | ESP 专用 Rust target / 工具链 | GitHub 预编译二进制 |
| **ldproxy** | `.cargo/config.toml` 中配置的 linker | `cargo install ldproxy` |

---

## 编译与烧录

### 识别串口（macOS）

```bash
ls /dev/cu.usb*
espflash board-info
```

`.cargo/config.toml` 已配置 `runner = "espflash flash --monitor"`，因此 `./cargo run` 等价于 build + 烧录 + 串口日志。

```bash
./cargo build                              # 主固件
./cargo build --examples                   # 全部示例
./cargo run                                # 烧录主固件
./cargo run --example 05_ble_csc           # 烧录指定示例
```

---

## 常见问题排查

| 现象 | 可能原因 | 处理 |
|------|----------|------|
| `cargo install` 极慢 / 失败 | 编译 OpenSSL | 改用 **Homebrew** 或 **GitHub 预编译 espup** |
| `brew install` 卡住 | Homebrew auto-update | `export HOMEBREW_NO_AUTO_UPDATE=1` |
| 找不到串口 | 驱动 / 线材 | 换数据线；`ls /dev/cu.*` |
| `espflash` 无法连接 | 端口占用 | 关闭其他串口监视器；`ESPFLASH_PORT=...` |
| 首次 `cargo build` 很久 | 下载并编译 ESP-IDF | 正常；`ESP_IDF_TOOLS_INSTALL_DIR=global` |
| `ldproxy` not found | PATH | 使用 `./cargo build` 或 `source scripts/env.sh` |
| BLE 扫不到 | Bluedroid / sdkconfig | 确认 `CONFIG_BT_NIMBLE_ENABLED=y`（见 `sdkconfig.defaults`） |
| OLED 无显示 | 接线 / 地址 | 3.3V、SDA/SCL；试对调 SDA/SCL；地址 0x3C/0x3D |
| 霍尔不计数 | 接线 / 边沿 | KY-003 S→GPIO4；磁铁靠近模块 LED 应亮；看 `02_hall_input` 日志 |

---

## Cursor / VS Code

推荐扩展：**rust-analyzer**。Rust target 为 `riscv32imc-esp-espidf`，IDE 可能提示部分 cfg 未知，可忽略或参考 [Rust on ESP Book](https://docs.espressif.com/projects/rust/book/)。

---

## 参考

- [esp-rs esp-idf-template](https://github.com/esp-rs/esp-idf-template)
- [Rust on ESP Book](https://docs.espressif.com/projects/rust/book/)
- [esp32-nimble](https://github.com/taks/esp32-nimble)
- CSC 规范：Bluetooth SIG Cycling Speed and Cadence Service `0x1816`
