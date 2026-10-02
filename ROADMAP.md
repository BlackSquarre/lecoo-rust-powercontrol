# Roadmap

This document records planned work for **Lecoo Rust PowerControl**. Items 1–3 are implemented in the current working source; item 4 remains planned. Current functionality and verification limits are documented in [the feature guide](docs/features.md).

## Planned work

### 1. Start with Windows — implemented

- Add an opt-in setting to launch the app when the current user signs in.
- Show whether automatic startup is enabled and let the user turn it off again.
- Choose an approach that works with the app's administrator requirement, and explain any Windows prompt during setup.
- Do not silently enable startup or leave stale startup entries after the user disables it.

**Acceptance checks:** startup runs only after the user enables it; the app starts with hardware access available; disabling the option prevents the next sign-in launch; repeated toggles do not create duplicate entries.

### 2. Notification-area icon and quick power controls — implemented

- Add a persistent icon in the Windows notification area (system tray), with a project-specific icon.
- Add a right-click menu for Quiet, Balanced, and Performance profiles.
- Mark the active profile in the menu, apply a selected profile, and report a failed switch clearly.
- Provide a way to open the main window and exit the app. Define whether closing the window minimizes to the tray and make that behavior discoverable.
- Avoid duplicate tray icons or app processes when the app is launched more than once.

**Acceptance checks:** the menu reflects the hardware's current profile; selecting a profile uses the same verified hardware-control path as the main UI; errors are visible; opening and exiting the app cleanly adds and removes the tray icon.

### 3. Fan-speed adjustment — implemented with readback limits

**Research update (2026-10-02):** The installed official binary confirms fan control uses WMI, while AMD temperature monitoring uses WinRing0. An independent Rust temperature probe has passed local sampling tests. A guarded WMI fan session is now implemented; exact duty/mode readback remains unavailable; see [driver and sensor research](docs/reference/wmi/native-sensor-research.md).

- First confirm which fan-control methods and safe ranges the compatible hardware firmware actually exposes.
- Add a fan control that reports the current measured RPM and exposes only supported adjustments, such as a safe manual target or supported fan mode.
- Validate every requested value against hardware limits and provide a clear way back to automatic/default control.
- Define safe behavior for communication failures, app exit, startup/shutdown, and high-temperature conditions before enabling manual control.
- If the firmware does not expose a safe writable control, report that clearly instead of presenting a nonfunctional control.

**Acceptance checks:** only supported values can be submitted; applied settings are read back and verified; invalid values are rejected; failures restore or retain a safe automatic/default setting; tests verify both control behavior and recovery on compatible hardware.

**Remaining acceptance limit:** the provider returns RPM but no target-duty or control-mode readback. Firmware responses and measured RPM are verified; this cannot fulfill exact setting readback. See [validation details](docs/testing.md).

### 4. Redesign the interface and add restrained motion

- Replace the current layout with an original visual design for Lecoo Rust PowerControl instead of copying the official control center's presentation.
- Reorder the main screen around the tasks users need most: current device status, power-profile selection, cooling status, then secondary system-resource details.
- Give fan controls a clear, dedicated place and keep less-used settings out of the primary control flow.
- Establish consistent typography, spacing, color, icons, and clear enabled/disabled/error states.
- Add short, purposeful transitions for page changes, profile selection, gauges, and notifications. Keep motion subtle and ensure it does not delay or obscure controls.

**Acceptance checks:** the primary status and profile controls are immediately apparent; layouts remain readable at supported window sizes; keyboard focus and contrast remain clear; animations do not block input and error states remain visible.

## Suggested implementation order

Keep this order unless hardware research changes the dependencies: startup preference, notification-area controls, fan-speed adjustment, then interface redesign and animation. Investigate the fan-control interface and safety limits before implementing it. Each feature should include its own validation and user-facing documentation before release.

---

## 中文

本文记录 **Lecoo Rust PowerControl** 的后续计划。当前源码已实现第 1–3 项；第 4 项 UI 重设计和动画仍保留为计划。具体用法与验证边界见[功能说明](docs/features.md)。

### 1. Windows 登录后自动启动 — 已实现

- 增加可选设置，由用户决定是否在当前用户登录 Windows 时启动应用。
- 显示自动启动状态，并允许用户随时关闭。
- 结合应用需要管理员权限的现状，选择合适的启动方式，并说明 Windows 显示的权限提示。
- 不默认静默开启；用户关闭后不得残留启动项。

**验收标准：** 只有用户启用后才会自动启动；启动后硬件访问仍然可用；关闭设置后下次登录不再启动；反复开关不会创建重复启动项。

### 2. 任务栏通知区域图标与右键快捷切换 — 已实现

- 在 Windows 任务栏通知区域（系统托盘）增加专属图标。
- 右键菜单提供安静、均衡、性能三种电源模式。
- 标识当前生效模式；选择模式后执行切换，失败时给出明确提示。
- 提供打开主窗口和退出应用的操作；明确关闭窗口是否缩到托盘，并让用户能找到该行为设置。
- 多次启动应用时避免出现重复进程或重复托盘图标。

**验收标准：** 菜单状态与硬件当前模式一致；快捷切换使用与主界面相同的硬件控制和读回验证流程；切换错误可见；打开或退出应用时托盘图标能正确创建和清理。

### 3. 风扇转速调节 — 已实现，读回能力受限

**调查进展（2026-10-02）：** 已从本机官方二进制确认风扇走 WMI、AMD 温度监控走 WinRing0，并完成独立 Rust 温度采样的本机验证。已接入带守护恢复的 WMI 风扇控制；固件仍不能读回实际目标占空比或控制模式，详见[驱动与传感器调查](docs/reference/wmi/native-sensor-research.md)。

- 先确认兼容设备固件实际提供哪些风扇控制接口及安全范围。
- 显示当前实测转速，并只开放硬件支持的调节方式，例如安全的手动目标值或固件支持的风扇模式。
- 按硬件限制校验输入，并提供恢复自动/默认控制的操作。
- 在启用手动控制前，明确处理通信失败、应用退出、登录/关机和高温等情况下的安全行为。
- 如果固件没有安全且可写的控制接口，应明确说明暂不支持，不显示无效控件。

**验收标准：** 只能提交硬件支持的值；应用会读回并验证设置；非法值会被拒绝；发生故障时恢复或保持安全的自动/默认状态；在兼容硬件上验证正常调节和恢复流程。

**尚未满足的验收限制：** 当前固件仅返回 RPM，没有目标占空比/控制模式读回接口。现已验证固件响应和实测转速，不能宣称精确设置读回验收完成，见[验证说明](docs/testing.md)。

### 4. 重新设计 UI 并加入克制的动画

- 为 Lecoo Rust PowerControl 设计原创界面，不照搬官方控制中心的视觉呈现。
- 重新安排主界面信息层级，优先展示设备状态和电源模式，其次是散热状态，再展示次要的系统资源信息。
- 为风扇控制安排清晰、独立的位置；不常用设置从主要操作区移出。
- 统一字体、间距、配色、图标，以及启用、禁用、错误等状态的视觉表现。
- 为页面切换、模式选择、仪表和提示加入短促且有意义的过渡动画；动画保持轻量，不延迟操作，也不遮挡控件。

**验收标准：** 打开应用后能立即看清主要状态和模式控件；常用窗口尺寸下内容清晰；键盘焦点和对比度明确；动画不阻塞输入，错误状态始终可见。

## 建议顺序

暂按以下顺序规划：登录启动设置、任务栏托盘快捷控制、风扇转速调节、UI 重设计和动画。风扇功能先完成接口和安全范围调查，再进入实现；UI 相关工作放在最后。每项功能发布前都要完成对应验证并补齐用户文档。
