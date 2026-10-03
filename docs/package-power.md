> 历史调查记录：v0.0.3 已移除 WinRing0 采样代码、探针和功率显示。当前温度使用 ACPI 热区 WMI；以下记录仅描述此前实现与实验。

# CPU Package 温度与功率

验证日期：2026-10-03；设备：AMD Ryzen 7 8745H。当前 GUI 的 CPU 温度来自 `src/platform/windows/sensors.rs`，通过现有 WinRing0 读取 AMD SMN `THM_TCON_TEMP`（`0x59800`）。OpenHardwareMonitor 将此源命名为 CPU Package；LibreHardwareMonitor 使用 Core (Tctl/Tdie) 命名。它不是所有核心温度的算术平均值。当前生产代码没有用未确认来源的 WMI 温度代替它。

## 功率读取

独立 Rust 功率采样能力现已接入紧凑 C 版 GUI，通过既有的一秒状态采样更新功率格。首次建立能量基线，失败或无效间隔显示不可用；后续有效采样恢复显示。当前源码的本地构建已包含此功能，已发布 v0.0.2 ZIP 尚未更新。

- 连接已经运行的 WinRing0，不安装或启动驱动，不加载 .NET 监控库。
- 使用 `IOCTL_OLS_READ_MSR`（`0x9C402084`）读取 AMD `0xC0010299` 能量单位和 `0xC001029B` 封装能量计数器；没有 MSR 写入或功率限制设置。
- 在已验证的单封装 CPU 上暂时绑定采样线程到逻辑 CPU 0，读取后恢复原线程亲和性。现有 CPU 型号和驱动版本校验仍适用。
- 用 `counter_delta × 2^(-ESU) / elapsed_seconds` 得到采样区间平均瓦数。时间来自单调时钟；32 位计数器回绕使用完整 wrapping subtraction。
- 拒绝单位变化、计数器不前进、间隔不足 100 ms 或超过 5 s、明显异常的结果；250 W 是本机原型的异常数据过滤上限，不是设备的功率限制。
- 开始采样时先建立基线，后续才产生功率值。接入监控时不需要阻塞等待，可在现有定时采样中保留前一个能量样本。

这是 CPU 的封装能量遥测，不是配置 TDP、PPT 限值，也不是插座测得的整机功耗。没有与外部功率仪器或同时运行的另一监控程序进行绝对精度校准。

## 本机实测

运行：

```powershell
cargo build --bin sensor_probe --locked --offline
# 在已提升权限的终端执行；驱动必须已经运行。
.\target\debug\sensor_probe.exe --power
```

不带参数仍保留原来的五次温度/WMI/RPM 对照采样。

本次功率探针连续 10 次约 500 ms 间隔采样均成功：能量单位寄存器 `0x00000000000A1003`，ESU=16，即每个计数约 15.259 微焦耳。当时负载下读数 13.638–26.064 W，平均 18.7876 W；温度 48.250–54.500°C。不是待机或满载基准测试。

本地报告：`logs/driver-research/package-power-verified.json`，包含原始计数、时间间隔、瓦数、温度和探针 SHA256。采样前后电源模式均为 1，驱动运行状态与定义相同。独立 PowerShell 重算计数器差值用于核对计算结果。库的 12 项单元测试通过，其中新增功率的单位换算、计数器回绕和异常样本测试。

## 资料

- [OpenHardwareMonitor AMD17CPU](https://github.com/openhardwaremonitor/openhardwaremonitor/blob/master/Hardware/CPU/AMD17CPU.cs)：CPU Package 温度源及 AMD Package 能量换算。
- [LibreHardwareMonitor Amd17Cpu](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor/blob/master/LibreHardwareMonitorLib/Hardware/Cpu/Amd17Cpu.cs)：Tctl/Tdie 命名与 Package 功率实现。
- [WinRing0 OlsIoctl.h](https://github.com/openhardwaremonitor/openhardwaremonitor/blob/master/External/WinRing0/OlsIoctl.h)：读取 MSR 的驱动协议。
