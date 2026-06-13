# 新设备上手指南

换电脑、重装系统或新同事加入项目时，按本文操作即可。

目标芯片：**ESP32-C3 SuperMini**  
工具链：**esp-idf-template**（Rust nightly + ESP-IDF v5.5.3）

---

## 流程概览

```text
克隆项目 → 安装依赖 → 加载 PATH → 编译验证
                                    ↓
                              连接 ESP32 → 按 example 逐级烧录验证
```

---

## 一、新电脑：开发环境

### 1. 克隆项目

```bash
git clone git@github.com:soulmate126/bike-cadence-sensor.git
cd bike-cadence-sensor
```

### 2. 安装 rustup（推荐）

若本机只有 Homebrew 的 `rust`，请安装 rustup。本项目依赖 **rustup nightly + build-std**，Homebrew cargo 会导致 `can't find crate for core` 错误。

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

安装后确认：

```bash
export PATH="$HOME/.cargo/bin:$PATH"
rustup toolchain install nightly --component rust-src
```

> 项目根目录的 `rust-toolchain.toml` 会在进入目录时自动选用 nightly。

### 3. 一键安装其余依赖

```bash
chmod +x scripts/install-deps.sh
./scripts/install-deps.sh
```

脚本会安装：

| 组件 | 来源 |
|------|------|
| cmake、ninja、python@3.12、espflash 等 | Homebrew（`Brewfile`） |
| espup | GitHub 预编译二进制 |
| ldproxy | `cargo install`（约 2 分钟） |

**首次编译**还会下载 ESP-IDF v5.5.3 并编译 C 组件，约 **10–30 分钟**，属正常现象。

### 4. 加载 PATH（必做）

Homebrew 与 rustup 同时存在时，**必须让 rustup 的 cargo 优先**。

项目已内置环境配置，**任选一种**：

| 方式 | 操作 |
|------|------|
| **项目 cargo 脚本（推荐）** | `./cargo build` — 自动加载 `scripts/env.sh` |
| 手动 source | `source scripts/env.sh`，之后可用 `cargo` |
| direnv | `brew install direnv`，`~/.zshrc` 加 `eval "$(direnv hook zsh)"`，进目录后 `direnv allow` |
| Cursor / VS Code | 打开本项目，集成终端读取 `.vscode/settings.json` |

自检：

```bash
./cargo --version
which cargo   # 使用 ./cargo 时内部会指向 ~/.cargo/bin/cargo
```

期望看到 **nightly** 版本，例如 `cargo 1.98.0-nightly (...)`。

### 5. 编译验证（无需插板）

```bash
./cargo build --example 01_blink
```

成功即表示新设备开发环境就绪。

---

## 二、ESP32 到手后：烧录验证

### 1. 连接开发板

- 使用**数据线**（非纯充电线）
- macOS 查看串口：

```bash
ls /dev/cu.usb*
# 常见：/dev/cu.usbmodem*
```

可选：确认芯片型号

```bash
espflash board-info
```

### 2. 指定串口（多设备时）

```bash
ESPFLASH_PORT=/dev/cu.usbmodem1101 ./cargo run --example 01_blink
```

### 3. 按顺序验证 example

建议严格按序进行，便于定位问题：

| 顺序 | 命令 | 验证内容 | 预期结果 |
|------|------|----------|----------|
| 1 | `./cargo run --example 01_blink` | 烧录 + 串口 + LED | GPIO8 LED 闪烁 |
| 2 | `./cargo run --example 02_hall_input` | 霍尔 GPIO | 磁铁靠近 GPIO4，边沿计数增加 |
| 3 | `./cargo run --example 03_ssd1306` | OLED | 屏幕显示 `Hello Bike` |
| 4 | `./cargo run --example 04_ble_advertise` | 仅广播 | 可扫到设备名，无 GATT |
| 5 | `./cargo run --example 05_ble_csc` | 霍尔 + BLE CSC | nRF Connect 订阅 0x2A5B 收到通知 |
| 6 | `./cargo run --example 06_hall_oled` | 霍尔 + OLED | 屏幕显示 RPM / COUNT |
| 7 | `./cargo run` | **主固件** | 霍尔 + OLED（可选）+ BLE + 状态 LED |

引脚：**霍尔 GPIO4**，**I2C GPIO5/6**，**LED GPIO8**。可调参数见 [`src/board/config.rs`](../src/board/config.rs)。

### 4. 手机验证 BLE CSC

**nRF Connect：**

1. 扫描 **DIY Cadence Sensor**
2. 连接 → 找到服务 **0x1816**（Cycling Speed and Cadence）
3. 对特征 **0x2A5B** 开启 Notify
4. 应收到 5 字节数据：`[flags, rev_lo, rev_hi, time_lo, time_hi]`（flags 含接触位）

`05_ble_csc` / 主固件使用真实霍尔输入；订阅 0x2A5B 后会**立即收到当前累计转数**。

**华为 GT5 Pro：**

```text
设置 → 健康与健身设备 → 添加设备 → 踏频器
锻炼 → 户外骑行/室内单车 → 运动准备阶段自动连接
```

固件已启用 **BLE Bonding**，首次升级后请删除并重新添加踏频器。

### 5. 主固件

```bash
./cargo run
```

```text
霍尔脉冲 → CadenceCalculator → CadenceData → CscServer::notify_measurement()
```

主固件特性：OLED 失败自动降级、GPIO8 状态 LED（未连接慢闪 / 已连接常亮 / 有踏频快闪）、断连后自动恢复广播。

### 6. 主机单元测试（无需板子）

```bash
cargo test -p cadence-core
```

---

## 三、最短路径（速查）

```bash
# 新电脑
git clone git@github.com:soulmate126/bike-cadence-sensor.git
cd bike-cadence-sensor
./scripts/install-deps.sh
./cargo build

# 有板子
./cargo run --example 01_blink
./cargo run --example 05_ble_csc
./cargo run
# → GT5 Pro：运动准备阶段自动连踏频器
```

---

## 四、常见问题

| 现象 | 原因 | 处理 |
|------|------|------|
| `can't find crate for core` | 用了 Homebrew 的 cargo | 改用 `./cargo build` 或 `source scripts/env.sh` |
| `ldproxy` not found | PATH 未包含 `~/.cargo/bin` | `source scripts/env.sh` |
| 首次 build 极慢 | 下载编译 ESP-IDF | 等待完成，确保网络稳定 |
| 找不到串口 | 驱动 / 线材 | 换数据线；`ls /dev/cu.*` |
| `espflash` 无法连接 | 端口被占用 | 关闭其他串口工具；设 `ESPFLASH_PORT` |
| BLE 扫不到 | sdkconfig / 内存 | 检查 `sdkconfig.defaults` 中 `CONFIG_BT_*` |
| `direnv allow` 无效 | shell 未配置 hook | `~/.zshrc` 加 `eval "$(direnv hook zsh)"` 后重开终端 |

---

## 五、相关文档

- [CSC 协议要点](csc-protocol.md)
- [README 完整说明](../README.md)
