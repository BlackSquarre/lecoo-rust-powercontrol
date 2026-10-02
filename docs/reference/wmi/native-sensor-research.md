# 官方监控驱动调查与独立 Rust 采样原型

调查及验证日期：2026-10-02；设备：Lecoo MINI PRO，AMD Ryzen 7 8745H。

## 结论

可以实现自己的用户态硬件访问代码。已完成独立 Rust 原型并在本机采样成功，不需要加载 `ControlCenter.exe`、Python 或官方目录中的 .NET DLL。

两个通道应分别处理：

- 风扇读取、风扇设置、电源模式：固件提供的 `root\WMI\PowerSwitchInterface`。
- 官方 AMD CPU 温度：OpenHardwareMonitor 通过 WinRing0 访问 AMD SMN。原型直接用 Windows API 访问现有驱动。

原型仍依赖**已经加载的 WinRing0 内核驱动**。这是独立用户态实现，不是自行编写的新内核驱动，也不代表移除官方驱动后仍可使用这条温度路径。现有 WMI 温度读取可以独立使用，但当前不能证明它与 CPU Package 是同一传感器。

## 本机文件和调用链证据

`C:\Program Files\ControlCenter\OpenHardwareMonitorLib.sys`：

| 项目 | 本次结果 |
| --- | --- |
| 文件描述 / 公司 | WinRing0 / OpenLibSys.org |
| 版本 / 大小 | 1.2.0.5 / 14544 字节 |
| 签名 | Valid；Noriyuki MIYAZAKI |
| SHA256 | `11BD2C9F9E2397C9A16E0990E4ED2CF0679498FE0FD418A3DFDAC60B5C160EE5` |
| 当前服务 | `WinRing0_1_2_0`；Running；Manual |
| 设备入口 | `\\.\WinRing0_1_2_0` |

通过 .NET **仅反射加载**读取本机 `OpenHardwareMonitorLib.dll` 的资源，内嵌 `OpenHardwareMonitor.Hardware.WinRing0x64.sys` 与磁盘上的 `.sys` 大小和 SHA256 完全一致。因此这不是专用的来酷风扇驱动，而是监控库释放出来的通用 WinRing0。

此外，直接解析本机 `ControlCenter.exe` 中的 PyInstaller Python 3.6 归档，静态反汇编 `CPUGPUMemorynew` 和主模块的风扇方法：

- 温度线程构造 `OpenHardwareMonitor.Hardware.Computer`，启用 CPU 并调用 `Open()` / `Update()`。
- AMD 分支使用该线程读到的传感器结果。本机程序会先转为整数，再计算 `int(temperature / 110 * 100)`；复刻时不应把这个显示缩放当作真实摄氏温度。
- 主模块的 `GetFanRPM`、`SetFan1Control`、`SetFan2Control` 直接调用 WMI，并未通过 `.sys` 操作风扇。
- `CPUTemperatureSpace.dll` 也是 OpenHardwareMonitor 的包装层，不提供独立内核协议。目录里另有 LibreHardwareMonitor，但本次确认的温度线程明确引用 OpenHardwareMonitor。

这次证据来自本机安装版本，不仅依赖此前第三方反编译源码。原始静态分析输出保存在本地忽略目录 `logs/driver-research/`；没有执行提取出的 Python 代码或 DLL 方法。

## 独立温度采样实现

代码：[`sensors.rs`](../../../src/platform/windows/sensors.rs)；入口：[`sensor_probe.rs`](../../../src/bin/sensor_probe.rs)。

公开 WinRing0 协议与本机 DLL IL 中的控制码一致：

| IOCTL | 用途 |
| --- | --- |
| `0x9C402000` | 读取驱动版本，原型要求 `0x01020005` |
| `0x9C406144` | 读取 PCI 配置 |
| `0x9C40A148` | 写入 PCI 配置；原型仅在内部用于 SMN 地址选择和恢复 |

本机 DLL 的 `AMD17CPU.ReadSmnRegister` 使用 PCI `0:0:0`：向配置偏移 `0x60` 写入 SMN 地址，再读取 `0x64`。温度地址为 `0x59800`，数值换算为 `((raw >> 21) & 0x7FF) / 8`；若 `raw & 0x80000` 非零，再减去 49°C。这与 [OpenHardwareMonitor AMD17CPU 源码](https://github.com/openhardwaremonitor/openhardwaremonitor/blob/master/Hardware/CPU/AMD17CPU.cs)相符；IOCTL 的结构见 [WinRing0 OlsIoctl.h](https://github.com/openhardwaremonitor/openhardwaremonitor/blob/master/External/WinRing0/OlsIoctl.h)。

原型目前只允许 Ryzen 7 8745H，并在采样前核对 PCI AMD 厂商标识。它共享官方 `Global\Access_PCI` 互斥锁；超时或锁被遗弃时拒绝采样。每次读取保存原地址选择值，在成功或失败后尝试恢复，并读回验证恢复；输出保留原始寄存器值。非法或明显异常的温度值会被拒绝。

温度采样包含**短暂的 PCI 地址选择写入**，不能称为完全没有硬件写入。它不设置 SMN 数据寄存器、风扇占空比、功率限制或电源模式。恢复机制覆盖函数正常返回的成功和错误路径，不保证进程被强制终止或机器断电时执行清理，当前已接入 GUI，采样失败显示不可用；手动风扇控制还由独立进程检查温度并提供心跳恢复。

## 风扇协议及尚未解决的边界

本机官方程序确认：

| 操作 | 参数或读数编码 |
| --- | --- |
| 读转速 | `GetFanControl(FanNumber=1)`；低 16 位为风扇 1 RPM，高 16 位为风扇 2 RPM |
| 自动 | `SetFanControl(FanNumber=1, FanDuty=101)` |
| 最大 | `FanDuty=100` |
| 自定义 | 官方界面范围 35–100%，切换自定义时先提交 50 |

这些是**官方程序的参数约定**，并非厂商保证的安全范围。本机 WMI 类没有直接读回“当前自动/手动模式及目标占空比”的方法；读取 RPM 不能等同于验证目标占空比。初次调查仅采样；后续实现的设置与恢复测试见[测试文档](../../testing.md)。

现已实现独立 WMI setter 与风扇守护会话；精确占空比读回仍无法满足。实机与故障注入验证区分处理 `ResultStatus` 的真实语义、各目标值的实际效果、恢复自动控制的效果，以及通信中断、应用崩溃、退出和过温时的行为。35% 不能直接当作固件已经证明的最低安全值。

温度采样与风扇控制代码保持分离，手动会话需要有效温度、定期心跳与自动恢复；对未经确认的设备拒绝写入。不要通过 WinRing0 扫描未知 EC 端口来代替已知 WMI 风扇接口。

## 本机验证

运行：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\test-sensors.ps1
```

脚本先构建诊断程序和单元测试，再请求 UAC，在管理员子进程中运行。只连接现有驱动，不安装、启动、停止或删除驱动服务；同时独立查询 WMI 电源模式和驱动状态，保存带二进制 SHA256 的 JSON 报告。

本次报告：`logs/driver-research/sensors-verified.json`：

- `Passed=true`；3 项测试通过：温度换算及范围位、异常值拒绝、IOCTL 编码。
- 原生读取连续 5 次成功：58.625、57.750、57.250、56.750、56.000°C。
- 同时 WMI 温度为 56–57，风扇为 2095–2143 RPM，固件报告 1 路风扇。
- 电源模式前后均为 1；驱动前后均为 Running / Manual，路径相同。
- 每次成功采样包含 SMN 地址选择恢复的读回检查。

这证明本机访问通道和独立换算实现能够工作，不是传感器绝对精度认证，也不验证手动风扇调节或其它 CPU 型号。

## 用户截图与显示逻辑复核

2026-10-02 用户提供两张官方界面截图，截图采集时间未给出：

- 风扇页选中“自动”，显示 2425 RPM，灰色滑杆旁显示 0%；当前页面仅显示一路风扇。
- 模式页突出显示“性能模式”，显示 2116 RPM、CPU 温度 49°C、磁盘 26%（253G/952G）、内存 68%（6.0G/8.8G）。两个页面读数来自不同截图，不能作为同一时刻的同步样本比较。
- 本机二进制的风扇自动模式回调会禁用滑杆，并向 WMI 提交 101。滑杆构造代码确认为最小 35、最大 100。因此自动模式下的 0% 标签不能作为实际风扇占空比或合法手动目标的证据。
- 再次核对温度绘制方法：AMD 采样线程的缩放结果进入 `_tempvalue`，动画更新 `_CPUTvalue`，最终文字直接显示 `_CPUTvalue` 并附加 C。缩放确实影响温度数字，而非仅影响圆环进度；在显示稳定时，49 对应的采样整数可能为 54。应以原生 CPU Package 读数为准，不能要求原型复刻这个缩放值。
- 安装目录的 `imgs/PerformenceMode.png` 与截图中央五边形图一致，这是模式说明的静态图片，不是温度、功耗或性能的实时测量图。

截图证实了控件呈现与显示读数，仍不验证风扇写入是否生效或故障后的自动恢复。下一步有信息价值的页面是“功能设定”“常规信息”和右上角菜单。
