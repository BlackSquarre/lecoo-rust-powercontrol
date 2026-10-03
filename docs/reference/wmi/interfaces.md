# WMI 接口开发参考

来源：既有审计报告、2026-10-02 管理员只读结果、本次 WMI 类元数据导出、公开反编译源码。

命名空间：`root/WMI`；类：`PowerSwitchInterface`。
实例：`ACPI\PNP0C14\IP3POWERSWITCH_0`，此前 Active=True。
各方法 ReturnValue 类型为 Boolean；另有输出参数 ResultStatus 时，应单独处理，不能把所有状态码解释为同一种成功码。
完整参数方向和类型以同目录 `PowerSwitchInterface-methods.txt` 为准。

| 方法 | 输入 → 输出 | 含义与验证 |
|---|---|---|
| GetPowerMode | 无 → CurrentPowerMode UInt8 | 实测 1；源码 0=均衡、1=性能、2=安静 |
| SetPowerMode | PowerMode UInt8 → ResultStatus UInt8 | 0/1/2 切换已在本机验证两轮 |
| GetFanControl | FanNumber UInt8 → FanDuty UInt32 | FanNumber=1 的低 16 位为风扇 1 RPM，高 16 位为风扇 2 RPM；2 号返回哨兵 |
| SetFanControl | FanNumber、FanDuty UInt8 → ResultStatus UInt8 | 已确认 101=自动、100=最大、35–100=手动目标；只返回 ResultStatus，没有目标模式/占空比读回 |
| GetFeatureValue | Reserved1..4 UInt8 → ResultStatus UInt32 | 编号与读数见下表，实测时 Reserved2..4=0 |
| SetFeatureValue | Reserved1..4 UInt8 → ResultStatus UInt8 | 源码开关使用编号 1/5/7 及 0/1；未写入测试 |
| GetHwTemp | HwTempType UInt8 → Temp UInt32 | 类型 0=2147483647，类型 1=49；位置与单位未确认 |
| GetKbdBltBrightness | BrightnessMode UInt8 → ResultStatus UInt8 | 类元数据存在，未调用；参数方向见实际导出 |
| SetKbdBltBrightness | BrightnessMode UInt8 → ResultStatus UInt8 | 未测试 |
| GetKbdLightMode | KbdLightMode UInt8 → ResultStatus UInt8 | 类元数据存在，未调用 |
| SetKbdLightMode | KbdLightMode UInt8 → ResultStatus UInt8 | 未测试 |
| GetKbdFnCtrl | KbdFnCtrl UInt8 → DeviceStatus UInt16 | 未测试 |
| SetKbdFnCtrl | KbdFnCtrl、DeviceStatus UInt8 → ResultStatus UInt8 | 未测试 |

## 功能编号

| Reserved1 | 源码用途 | 实测 ResultStatus | 解释边界 |
|---:|---|---:|---|
| 1 | OSD 状态/开关 | 1 | 写入源码使用 0/1，未测效果 |
| 2 | 模式数量 | 3 | 只读查询 |
| 3 | 风扇数量 | 1 | 只读查询 |
| 4 | GPU/第二风扇能力 | 15 | 源码仅判断是否等于 1，15 含义未知 |
| 5 | CPU Turbo 能力/开关 | 255 | 255 含义未知，不应判定为已开启 |
| 7 | 灯光模式 | 255 | 255 含义未知 |

`2147483647` 可能是无效/哨兵值，建议作为“未知”显示并保留原始值；其确切约定无厂商文档。
没有发现 UMA/显存容量设置接口。不要用未知 Reserved 编号尝试探测写入。

## 其它本地接口

`DllCallMethod.dll` 导出：EyeModeNormal、GETNightMode、GETProtectEyeMode、NightModeNormal、NormalMode、SayHello。
源码将部分函数用于护眼/夜间模式；函数签名和写入效果未在本机调用验证。
ImagesShowService 使用 LocalSystem 自动运行；内部 IPC 未解析。
WinRing0_1_2_0 是官方软件的现有监控驱动；本应用 v0.0.3 已移除对它的访问，也不附带、安装或卸载它。此前独立 Rust 采样结果保留在[历史调查](native-sensor-research.md)。

当前显示温度来自 `root\WMI:MSAcpi_ThermalZoneTemperature`，只选择已观察到的 `ACPI\ThermalZone\TZ01_0`。`CurrentTemperature` 按 0.1 K 转为摄氏：`raw / 10 - 273.15`；0、异常范围、访问失败或缺少该实例均显示不可用。本机 raw=3112 对应 38.05°C，未验证该热区与 CPU 封装温度的对应关系。电源模式和风扇控制仍使用独立编写的 PowerSwitchInterface WMI 调用。

## 建议的实现顺序

1. 枚举 Active 的 WMI 实例，读取模式和能力值，保存原始返回值。
2. 展示未知值和访问拒绝，不把值异常当作可写功能已支持。
3. 单独处理需要提权的硬件调用，按功能限制可调用方法和参数。
4. 写入验证单独记录前值、输入、返回和后值；后续模式切换验证见 [自动测试说明](../../testing.md)。

