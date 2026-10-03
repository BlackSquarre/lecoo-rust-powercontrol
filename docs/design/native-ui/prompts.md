# Native Windows UI sketches

Generated with the built-in GPT Image tool. These are proposals, not running UI screenshots.

Final correction to C: Edit the provided Windows utility UI layout sketch. Keep the entire window, dimensions, arrangement, all Chinese text, all other controls, colors, typography and whitespace unchanged. Make only one correction: in the center "风扇控制" section, the radio-option row currently shows "自动" and "手动"; add the missing THIRD native radio option labeled exactly "最大" at the right of "手动", on the same row and within the center column. Three options should read "自动" "手动" "最大"; keep "手动" selected and other two unselected. Preserve the 50% slider and "松开滑条后自动设置", no Apply button. This third maximum mode is essential. Do not remove, alter or relabel any other features.
Fan slider: commit the selected percentage on release; no Apply button. Keyboard adjustments should be coalesced before committing. Preserve 35–100% validation, live temperature checks and worker recovery.

## 方案 A · 双栏仪表盘

```text
Use case: ui-mockup. Create ONE clean, high-quality UI layout sketch for a real Windows desktop utility named "Lecoo Rust PowerControl", in Simplified Chinese. This is a design proposal for implementation in Rust using Win32 native controls, prioritizing minimal memory usage. Use realistic Windows 11 standard title bar with minimize/maximize/close, Segoe UI / Microsoft YaHei UI typography, native radio buttons, native checkboxes, standard push buttons, native horizontal trackbar slider and slim progress bars. Minimal, refined, generously spaced but information-dense; matte off-white surfaces, dark charcoal text, light gray separators, subtle teal-blue system selection accent. No photos, glossy effects, acrylic, glass blur, neon, 3D, custom radial gauges, animated charts or decorative fan illustrations. Straight-on flat application screenshot, no laptop, no perspective, no branding outside window. Precise legible Chinese text and consistent alignment.

ALL existing functionality must be represented visibly in this one window:
1. Live status: "CPU 温度" "49 °C", "风扇转速" "2116 RPM", "内存" "68% · 6.0 / 8.8 GiB", "磁盘" "26% · 253 / 952 GiB". Use small progress bars for memory and disk.
2. "电源模式" native radio group: "安静" "均衡" "性能"; "性能" selected, subtle status "当前：性能模式".
3. "风扇控制" native radio group: "自动" "手动" "最大"; "手动" selected. Horizontal native slider at 50%, range labeled "35%" and "100%", prominent value "50%"; caption "松开滑条后自动设置". DO NOT include any Apply button; releasing the slider commits automatically. Small status "本次请求：50% · 连接正常", subtle standard "重新连接" button. Small safety note "85°C 或通信中断时恢复自动"; "固件仅提供 RPM，实际占空比不可读回".
4. "设置" native unchecked checkboxes "登录后启动（进入托盘）" and "关闭窗口时缩到托盘", status "登录启动：已关闭", small native button "刷新状态".
5. System/about information "Lecoo MINI PRO-AHP · Ryzen 7 8745H" and "原生 Windows 界面 · v0.0.2". Small note "最小化后释放界面，后台保护继续运行".
6. Bottom native buttons "收进托盘" "退出". A tiny informational footer "托盘支持模式切换，双击打开窗口".
Feature labels and controls should feel like an implementable utility, with excellent visual hierarchy, readable compact text, and subtle error/status area saying "设备连接正常". No invented features.
Layout A: ONE landscape window about 900 x 720 logical pixels. Place four equal lightweight status tiles across the top, using simple native STATIC text and progress bars in thin border groups, no shadows. Beneath, a broad left column (60%) contains power mode then fan controls, a narrow right column (40%) contains startup/preferences then device information. Bottom full-width thin footer with tray/exit buttons aligned right. Major grouping through whitespace and tiny headings. Add a small unobtrusive annotation OUTSIDE the window at top-left reading "A / 双栏仪表盘". Render the entire window and annotation with margins in a landscape canvas.
```

## 方案 B · 紧凑纵向面板

```text
Use case: ui-mockup. Create ONE clean, high-quality UI layout sketch for a real Windows desktop utility named "Lecoo Rust PowerControl", in Simplified Chinese. This is a design proposal for implementation in Rust using Win32 native controls, prioritizing minimal memory usage. Use realistic Windows 11 standard title bar with minimize/maximize/close, Segoe UI / Microsoft YaHei UI typography, native radio buttons, native checkboxes, standard push buttons, native horizontal trackbar slider and slim progress bars. Minimal, refined, generously spaced but information-dense; matte off-white surfaces, dark charcoal text, light gray separators, subtle teal-blue system selection accent. No photos, glossy effects, acrylic, glass blur, neon, 3D, custom radial gauges, animated charts or decorative fan illustrations. Straight-on flat application screenshot, no laptop, no perspective, no branding outside window. Precise legible Chinese text and consistent alignment.

ALL existing functionality must be represented visibly in this one window:
1. Live status: "CPU 温度" "49 °C", "风扇转速" "2116 RPM", "内存" "68% · 6.0 / 8.8 GiB", "磁盘" "26% · 253 / 952 GiB". Use small progress bars for memory and disk.
2. "电源模式" native radio group: "安静" "均衡" "性能"; "性能" selected, subtle status "当前：性能模式".
3. "风扇控制" native radio group: "自动" "手动" "最大"; "手动" selected. Horizontal native slider at 50%, range labeled "35%" and "100%", prominent value "50%"; caption "松开滑条后自动设置". DO NOT include any Apply button; releasing the slider commits automatically. Small status "本次请求：50% · 连接正常", subtle standard "重新连接" button. Small safety note "85°C 或通信中断时恢复自动"; "固件仅提供 RPM，实际占空比不可读回".
4. "设置" native unchecked checkboxes "登录后启动（进入托盘）" and "关闭窗口时缩到托盘", status "登录启动：已关闭", small native button "刷新状态".
5. System/about information "Lecoo MINI PRO-AHP · Ryzen 7 8745H" and "原生 Windows 界面 · v0.0.2". Small note "最小化后释放界面，后台保护继续运行".
6. Bottom native buttons "收进托盘" "退出". A tiny informational footer "托盘支持模式切换，双击打开窗口".
Feature labels and controls should feel like an implementable utility, with excellent visual hierarchy, readable compact text, and subtle error/status area saying "设备连接正常". No invented features.
Layout B: ONE compact portrait-oriented window about 580 x 850 logical pixels. No sidebar, no card grid or left/right settings column. Live temperatures and RPM share the top line with large quiet numeric typography; memory and disk are two simple horizontal status rows underneath. Then stack power mode group, fan group with wide slider, settings checkboxes and device information vertically using horizontal hairline separators. Keep all functionality visible, fit reasonably without scrolling. Bottom tray and exit buttons. Tight practical density with comfortable spacing. Add a small unobtrusive annotation OUTSIDE the window at top-left reading "B / 紧凑纵向面板". Render full window with margins on a portrait canvas.
```

## 方案 C · 横向工作台

```text
Use case: ui-mockup. Create ONE clean, high-quality UI layout sketch for a real Windows desktop utility named "Lecoo Rust PowerControl", in Simplified Chinese. This is a design proposal for implementation in Rust using Win32 native controls, prioritizing minimal memory usage. Use realistic Windows 11 standard title bar with minimize/maximize/close, Segoe UI / Microsoft YaHei UI typography, native radio buttons, native checkboxes, standard push buttons, native horizontal trackbar slider and slim progress bars. Minimal, refined, generously spaced but information-dense; matte off-white surfaces, dark charcoal text, light gray separators, subtle teal-blue system selection accent. No photos, glossy effects, acrylic, glass blur, neon, 3D, custom radial gauges, animated charts or decorative fan illustrations. Straight-on flat application screenshot, no laptop, no perspective, no branding outside window. Precise legible Chinese text and consistent alignment.

ALL existing functionality must be represented visibly in this one window:
1. Live status: "CPU 温度" "49 °C", "风扇转速" "2116 RPM", "内存" "68% · 6.0 / 8.8 GiB", "磁盘" "26% · 253 / 952 GiB". Use small progress bars for memory and disk.
2. "电源模式" native radio group: "安静" "均衡" "性能"; "性能" selected, subtle status "当前：性能模式".
3. "风扇控制" native radio group: "自动" "手动" "最大"; "手动" selected. Horizontal native slider at 50%, range labeled "35%" and "100%", prominent value "50%"; caption "松开滑条后自动设置". DO NOT include any Apply button; releasing the slider commits automatically. Small status "本次请求：50% · 连接正常", subtle standard "重新连接" button. Small safety note "85°C 或通信中断时恢复自动"; "固件仅提供 RPM，实际占空比不可读回".
4. "设置" native unchecked checkboxes "登录后启动（进入托盘）" and "关闭窗口时缩到托盘", status "登录启动：已关闭", small native button "刷新状态".
5. System/about information "Lecoo MINI PRO-AHP · Ryzen 7 8745H" and "原生 Windows 界面 · v0.0.2". Small note "最小化后释放界面，后台保护继续运行".
6. Bottom native buttons "收进托盘" "退出". A tiny informational footer "托盘支持模式切换，双击打开窗口".
Feature labels and controls should feel like an implementable utility, with excellent visual hierarchy, readable compact text, and subtle error/status area saying "设备连接正常". No invented features.
Layout C: ONE wide short landscape window about 1080 x 610 logical pixels, clearly different from dashboard and portrait. Native-looking slim left navigation rail, 130 logical pixels wide, with four simple text buttons "总览" "风扇" "设置" "信息", "总览" selected using a subtle system highlight. Everything required is visible in this overview: main workspace center has a compact two-row status strip spanning the top; lower content arranged as three vertical zones, left power-mode radio controls vertically stacked, middle larger fan-control area with the long slider, right compact startup/preferences and device information. Dividers and spacing rather than decorative rounded cards. Bottom shared status strip, tray and exit at right. Add a small unobtrusive annotation OUTSIDE the window at top-left reading "C / 横向工作台". Render the full window and annotation with margins on a wide landscape canvas.
```

