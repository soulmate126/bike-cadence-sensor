# BLE CSC 协议要点

> Cycling Speed and Cadence Service — Bluetooth SIG

本项目让 **华为 GT5 Pro** 通过标准 CSC 协议识别自制踏频器（已验证）。

## 核心认知

CSC **不发送 RPM**。发送的是：

- **累计曲柄转数**（Cumulative Crank Revolutions）
- **上次事件时间**（Last Crank Event Time）
- **传感器接触状态**（可选 flags bit2/3）

手表 / 手机收到后，用两次通知之间的时间差和转数差**自行计算 RPM**。

## UUID

| 名称 | UUID |
|------|------|
| CSC Service | `0x1816` |
| CSC Measurement | `0x2A5B` |
| CSC Feature | `0x2A5C` |
| Sensor Location | `0x2A5D` |
| CCCD | `0x2902` |

## CSC Measurement 数据格式

### Flags（1 字节）

| Bit | 含义 |
|-----|------|
| 0 | Wheel Revolution Data Present |
| 1 | Crank Revolution Data Present |
| 2 | Sensor Contact Supported |
| 3 | Sensor Contact Detected |

本项目：`0x06`（无接触）或 `0x0E`（磁铁贴近）

### 仅踏频时（5 字节）

```text
[Flags: u8]
[Cumulative Crank Revolutions: u16 LE]
[Last Crank Event Time: u16 LE]
```

### Sensor Location（0x2A5D）

单字节枚举，本项目为 **5 = Left Crank**。

### 事件时间

- 单位：**1/1024 秒**
- 类型：`u16`，会回绕（约 64 秒后溢出）
- 接收端用两次值的差值计算间隔（需处理回绕）

## BLE 连接优化

| 能力 | 说明 |
|------|------|
| **Bonding** | `AuthReq::Bond` + NVS 持久化，便于手表自动重连 |
| **订阅即 notify** | 客户端开启 0x2A5B Notify 后立即发送当前转数 |
| **Preferred MTU 247** | 由中心端完成交换；CSC 载荷仅 5 字节 |
| **断连恢复广播** | NimBLE `advertise_on_disconnect` + ESP32-C3 `on_complete` |

## 与 RPM 的关系

```text
RPM = (delta_revolutions / delta_time_seconds) * 60
```

其中 `delta_time_seconds = delta_csc_time / 1024`

## 代码映射

```text
HallSensor (GPIO4 中断) → CadenceCalculator → CadenceSnapshot
CadenceData::from_snapshot()   → CSC 数据结构（含接触位）
CadenceData::encode()          → 通知字节流
CscServer::notify_measurement()→ BLE GATT 通知
```

实现文件：

- `crates/cadence-core/` — RPM 算法（主机可测）
- `src/sensors/hall.rs` — 霍尔驱动
- `src/ble/csc.rs` — CSC 数据编码
- `src/ble/server.rs` — NimBLE GATT Server
- `src/board/config.rs` — 去抖 / 超时等可调参数

## GT5 Pro 使用路径

```text
1. 设置 → 健康与健身设备 → 添加设备 → 踏频器
2. 锻炼 → 户外骑行 / 室内单车
3. 运动准备界面等待自动连接（无需每次手动点「重新连接」）
4. 设置 → 锻炼 → 手机查看数据（如需手机同步踏频）
```

升级固件启用 Bonding 后，需**删除并重新添加**踏频器一次。

## 参考

- [Bluetooth SIG CSC Service](https://www.bluetooth.com/specifications/specs/cycling-speed-and-cadence-service-1-0/)
- [esp32-nimble Bonding 说明](https://github.com/taks/esp32-nimble)
