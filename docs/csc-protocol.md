# BLE CSC 协议要点

> Cycling Speed and Cadence Service — Bluetooth SIG

本项目目标是让 **华为 GT5 Pro** 通过标准 CSC 协议识别自制踏频器。

## 核心认知

CSC **不发送 RPM**。发送的是：

- **累计曲柄转数**（Cumulative Crank Revolutions）
- **上次事件时间**（Last Crank Event Time）

手表 / 手机收到后，用两次通知之间的时间差和转数差**自行计算 RPM**。

## UUID

| 名称 | UUID |
|------|------|
| CSC Service | `0x1816` |
| CSC Measurement | `0x2A5B` |
| CSC Feature | `0x2A5C` |
| Sensor Location | `0x2A5D` |

## CSC Measurement 数据格式

### Flags（1 字节）

| Bit | 含义 |
|-----|------|
| 0 | Wheel Revolution Data Present |
| 1 | Crank Revolution Data Present |
| 2 | Sensor Contact Supported |
| 3 | Sensor Contact Detected |

本项目仅踏频：`Flags = 0x02`

### 仅踏频时（5 字节）

```text
[Flags: u8]
[Cumulative Crank Revolutions: u16 LE]
[Last Crank Event Time: u16 LE]
```

### 事件时间

- 单位：**1/1024 秒**
- 类型：`u16`，会回绕（约 64 秒后溢出）
- 接收端用两次值的差值计算间隔（需处理回绕）

转换：`csc_time = (timestamp_ms * 1024 / 1000) as u16`

## 与 RPM 的关系

```text
RPM = (delta_revolutions / delta_time_seconds) * 60
```

其中 `delta_time_seconds = delta_csc_time / 1024`

## 代码映射

```text
CadenceCalculator::on_pulse()  → 累计转数 + 时间戳
CadenceData::from_snapshot()   → CSC 数据结构
CadenceData::encode()          → 通知字节流
CscServer::notify_measurement()→ BLE GATT 通知
```

实现文件：

- `src/cadence/mod.rs` — RPM 算法
- `src/ble/csc.rs` — CSC 数据编码
- `src/ble/server.rs` — GATT Server 骨架
- `examples/05_ble_csc.rs` — 模拟踏频 + BLE 通知

## GT5 Pro 配对路径（待设备验证）

```text
设置 → 健康与健身设备 → 添加设备 → 踏频器
```

## 参考

- [Bluetooth SIG CSC Service](https://www.bluetooth.com/specifications/specs/cycling-speed-and-cadence-service-1-0/)
- 项目实现：`src/ble/csc.rs`、`src/cadence/mod.rs`
