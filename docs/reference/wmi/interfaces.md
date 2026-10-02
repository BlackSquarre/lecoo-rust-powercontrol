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
WinRing0_1_2_0 是现有硬件监控驱动。本机二进制调查已确认官方 AMD 温度经 OpenHardwareMonitor 访问它，风扇设置则走 WMI。现已实现并在 Ryzen 7 8745H 上验证独立 Rust 温度采样原型，详见[驱动调查及采样验证](native-sensor-research.md)。原生采样复用已加载的驱动，未实现新内核驱动；现已接入带独立守护进程的 WMI 风扇控制，边界见[功能说明](../../features.md)。

## 建议的实现顺序

1. 枚举 Active 的 WMI 实例，读取模式和能力值，保存原始返回值。
2. 展示未知值和访问拒绝，不把值异常当作可写功能已支持。
3. 单独处理需要提权的硬件调用，按功能限制可调用方法和参数。
4. 写入验证单独记录前值、输入、返回和后值；后续模式切换验证见 [自动测试说明](../../testing.md)。

