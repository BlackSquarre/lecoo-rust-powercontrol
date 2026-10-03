# Lecoo Rust PowerControl

<p align="center">
  <strong>使用 Rust 构建的来酷 Windows 控制中心。</strong><br>
  通过使用 Rust 构建的轻量级 Windows 桌面应用切换电源模式并查看硬件状态。
</p>

<p align="center">
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases"><img src="https://img.shields.io/github/v/release/BlackSquarre/lecoo-rust-powercontrol?display_name=tag" alt="最新版本"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml/badge.svg" alt="Windows 发布构建"></a>
  <img src="https://img.shields.io/badge/Rust-2021-orange?logo=rust" alt="Rust 2021">
  <img src="https://img.shields.io/badge/platform-Windows%20x64-blue" alt="Windows x64">
</p>

**作者：** [BlackSquarre](https://github.com/BlackSquarre) · **当前版本：** [v0.0.3](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.0.3) · **Language:** [English](README.md)

Lecoo Rust PowerControl 是独立社区项目，与来酷或联想没有隶属、合作或背书关系。

**当前实测设备：** Lecoo MINI PRO-AHP · **CPU：** AMD Ryzen 7 8745H

## 功能

使用紧凑的 Windows 原生界面，支持系统深浅色主题、中英文切换、关于窗口和可记住的关闭选择；见[界面说明](docs/native-ui.md)。

- 切换**安静、均衡、性能**三种电源模式。
- 切换后重新读取硬件状态，确认模式是否生效。
- 查看实测风扇 RPM 和 ACPI 热区温度，以及系统内存、磁盘使用情况。
- 图形界面和硬件测试工具共用同一套 Rust 硬件控制代码。

访问硬件需要管理员权限和兼容的来酷 WMI 接口。提供可选登录启动、托盘快捷模式与单实例，以及自动/最大风量控制。热区温度未经 CPU 保护验证，降低风量的手动目标暂时禁用。应用不包含或访问 WinRing0，不需要另装传感器驱动、.NET 或 VC++ 运行库。固件仅报告实测 RPM，不能读回占空比或控制模式；见[功能说明](docs/features.md)。

## 下载

在 [v0.0.3 Release](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.0.3) 下载 Windows x64 版本：

- `lecoo-rust-powercontrol-v0.0.3-windows-x64.zip`：控制中心、硬件测试工具和可执行文件校验值。
- `lecoo-rust-powercontrol-v0.0.3-windows-x64-setup.exe`：安装版，包含卸载程序。

开机自动运行可在应用设置中启用“登录后启动（进入托盘）”，默认关闭。

以管理员身份运行 `lecoo-control-center.exe` 才能访问硬件控制。验证详情和已知限制见 [Release 日志](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.0.3)。

## 从源码构建

应用面向 64 位 Windows。请安装 stable Rust MSVC 工具链，以及包含 Windows SDK 的 Visual Studio Build Tools，然后运行：

```powershell
cargo fetch --locked
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-release.ps1
```

打包还需要 Inno Setup 6。脚本在 `dist/` 生成 ZIP 与安装包；GitHub Actions 对每个版本标签生成两种格式，并检查安装及卸载。原生 C 运行库已静态链接。

## 自动硬件测试

在兼容的 Windows 设备上运行：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\test-power-modes.ps1
```

脚本会请求管理员权限，使用独立的 Windows CIM 读取核对三种电源模式，验证非法模式值会被拒绝，并在成功或失败后恢复原模式。添加 `-ReadOnly` 可只读取硬件状态，不切换电源模式。详细 JSON 报告写入 `logs/power-mode-tests/`；该目录已从公开仓库排除。

参数和恢复行为见[测试文档](docs/testing.md)，硬件接口说明见 [WMI 接口文档](docs/reference/wmi/interfaces.md)。

## 后续计划

后续功能规划见[路线图](ROADMAP.md)。登录启动、托盘、原生界面与自动/最大风量已实现；动画暂缓以保持轻量。

## 项目结构

```text
src/       Rust 应用、界面、硬件抽象和测试工具
scripts/   Windows 构建与自动硬件测试脚本
docs/      环境准备、测试和硬件接口文档
.github/   Windows 发布工作流
```

## 许可证

本项目未授予开源许可证。代码版权归 BlackSquarre 所有；仓库公开可见不代表允许转载或复用代码。
