# 自动硬件测试

入口：`scripts/test-power-modes.ps1`。测试程序 `src/bin/hardware_test.rs` 和 GUI 共用 `src/lib.rs` 中的生产硬件控制代码。
测试要求本机 WMI 接口存在且有管理员权限；脚本可自动请求 UAC。

## 参数

| 参数 | 用途 |
| --- | --- |
| `-Cycles 1..10` | 三种模式切换的轮数，默认 1 |
| `-ReadOnly` | 只读取状态，不执行写入或非法值测试 |
| `-SkipBuild` | 使用当前 `dist` 中的发布程序 |
| `-ReportPath 路径` | 指定 JSON 报告，否则使用 `logs/power-mode-tests` 下的带时间文件名 |
| `-FailAfterFirstSwitch` | 第一次切换后故意失败，用于验证恢复；预期退出码 1 |

## 流程

1. 默认调用 `scripts/build-release.ps1` 构建并发布两个程序到 `dist`。
2. 独立 CIM 查询记录原始模式，并核对生产代码的读取结果和模式数量。
3. 普通测试拒绝非法值 3，确认模式未改变。
4. 每轮按安静(2)、均衡(0)、性能(1) 切换，分别检查生产代码和独立 CIM 的实际读回。
5. 在 finally 中恢复原始模式，独立读回验证恢复结果。生产恢复失败时尝试 CIM 恢复，整次测试仍判失败。

电源模式脚本每次调用还读取风扇、WMI 温度、模式数量和风扇数量；手动风扇测试由下述独立功能脚本完成。
不要在测试时同时用其它程序切换模式。

退出码 0 表示全部通过，1 表示失败。报告包含二进制哈希、各次调用输出、模式转换记录和恢复状态。

## 已验证结果

- [两轮共 6 次模式切换](../logs/power-mode-tests/power-modes-verified.json)：Passed=true，Restored=true。
- [故意失败后的恢复](../logs/power-mode-tests/power-modes-recovery.json)：按预期失败，从 1 切换为 2 后恢复为 1。
- [最初只读检查](../logs/power-mode-tests/power-modes-readonly.json)：Passed=true，无写入。

这些报告记录的是保存时的二进制 SHA256；后续重新编译产生的二进制应重新测试。

## 独立传感器采样实验

当前源码的原生界面实机验证入口为 `scripts/test-native-ui.ps1`；它会暂时修改电源模式、应用关闭偏好与系统应用深浅色主题，在 finally 中恢复。验证滑条自动提交、关闭对话框期间的心跳、记住/重置/取消、标题栏和客户区主题切换，以及反复销毁/重建窗口时的 GDI/USER 资源。详情见[当前界面说明](native-ui.md)。

`scripts/test-sensors.ps1` 构建并运行 `sensor_probe` 和传感器单元测试；使用已加载的 WinRing0 驱动，在 Ryzen 7 8745H 上核对原生 CPU 温度采样，同时记录 WMI 转速、温度和电源模式。脚本请求管理员权限，报告写入 `logs/driver-research`。它不包含手动风扇调节测试，温度采样需要短暂改变并恢复 PCI SMN 地址选择寄存器。

本机 5 次采样与 3 项单元测试已通过；范围和限制见[调查文档](reference/wmi/native-sensor-research.md)。该诊断程序暂不加入正式发行包。

## 登录启动、托盘与风扇验证

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\test-features.ps1
```

默认构建调试程序和库单元测试，再在管理员子进程中验证。`-SkipBuild -Executable 完整路径 -UnitTestExecutable 完整路径` 可以验证已构建的发布程序；`-ReportPath` 指定报告，否则保存到 `logs/feature-tests`。测试前关闭本项目应用；脚本不关闭官方 Control Center。

验证包括：

1. 冷却状态机与温度换算单元测试：非法输入、过温、采样失效、固件拒绝、通信错误、心跳超时、退出自动恢复、自动恢复重试和最大风量备用请求。
2. 登录任务重复启用与删除、读取实际定义（当前用户、登录触发器、InteractiveToken、HighestAvailable）、手动运行已注册任务并确认只启动一个应用；最终恢复本应用原有任务或删除测试任务。不执行注销，真实登录触发仍未现场验证。
3. GUI 冒烟测试验证 Windows 确认托盘注册/更新、隐藏/打开、第二次启动唤醒、风扇请求、隐藏后持续心跳和退出。使用应用内同一动作处理路径；不模拟鼠标点击原生右键菜单，物理菜单交互需人工确认。
4. 同一生产风扇守护进程短时提交 35/50/100 目标，独立 CIM 记录实测 RPM，检查最大风量比低目标增加；非法 34 拒绝；超过 3 秒停止心跳、关闭父管道均检查自动恢复响应与守护退出。
5. finally 独立 CIM 请求自动，记录响应和非零 RPM，并确认电源模式未改变。测试开始后会以自动控制结束；固件无法恢复一个未知的外部手动目标。

本次初始实机功能报告 `logs/feature-tests/features-verified.json` 已通过，35/50/100 目标下短时读数为 2041/2658/3809 RPM，心跳恢复响应编码为 101，EOF 退出返回 STOPPED。最终发布构建的验证报告另存为 `features-release.json`。报告包含精确二进制 SHA256，后续修改后应重新验证。

2026-10-02 的发布构建复核：`features-release.json` 中 `Passed=true`、`Restored=true`，10 项单元测试通过，35/50/100 的独立读数为 2026/2645/3743 RPM；窗口隐藏超过 3 秒仍保持 50 目标心跳，隐藏时退出后自动恢复。`logs/power-mode-tests/power-modes-features-release.json` 的三种模式读回也通过，原始模式 1 已恢复。两份报告中的应用 SHA256 与 `dist/SHA256SUMS.txt` 一致；测试后登录启动任务不存在。

**验证边界：** RPM 变化证明目标请求影响风扇，但不证明实际占空比或自动/手动模式；固件没有这些读回接口。过温、通讯失败和恢复失败采用故障注入测试，不人为加热设备、卸载驱动或切断固件通信。自动恢复有请求响应与 RPM 读回，不能声称精确模式验证或所有负载下安全。限制和使用说明见[功能文档](features.md)。


## 紧凑 C 版与中英文界面

`powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test-native-ui.ps1` 现包含中英文切换、进程重启后的语言记忆、实时 CPU Package 功率、设置/关于/第三方声明、当前年份、弹窗期间手动风扇心跳和八次界面销毁重建。报告与实际截图见本地 `logs/compact-ui/`。测试临时修改语言偏好、系统应用主题、电源模式及风扇目标，结束后恢复偏好、主题与模式，并请求风扇自动控制。应先正常关闭正在使用的应用。

2026-10-04 的 `native-ui-final.json` 与 `features-final.json` 通过，均恢复测试环境且对应同一二进制 SHA256；18 项单元测试通过。中英术语见 [localization.md](localization.md)。
