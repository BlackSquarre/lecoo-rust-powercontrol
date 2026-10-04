# Lecoo Rust PowerControl

<p align="center">
  <strong>使用 Rust 构建的轻量级 Windows 控制中心，为兼容的来酷设备切换电源模式并查看硬件状态。</strong>
</p>

<p align="center">
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases"><img src="https://img.shields.io/github/v/release/BlackSquarre/lecoo-rust-powercontrol?display_name=tag" alt="最新版本"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml/badge.svg" alt="Windows 发布构建"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/ci.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/ci.yml/badge.svg" alt="Windows CI"></a>
  <img src="https://img.shields.io/badge/Rust-2021-orange?logo=rust" alt="Rust 2021">
  <img src="https://img.shields.io/badge/platform-Windows%20x64-blue" alt="Windows x64">
</p>

<p align="center">
  <a href="README.md">English</a> | 简体中文 | <a href="README.zh-Hant.md">繁體中文</a> | <a href="README.ja.md">日本語</a> | <a href="README.ko.md">한국어</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.de.md">Deutsch</a>
</p>

**作者：** [BlackSquarre](https://github.com/BlackSquarre) · **当前版本：** [v0.1.0](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.1.0)

> 本项目为独立社区项目，与来酷或联想没有隶属、合作或背书关系。

**当前实测设备：** Lecoo MINI PRO-AHP · **CPU：** AMD Ryzen 7 8745H

## 功能

- 切换**安静、均衡、性能**三种电源模式，并在切换后重新读取硬件状态。
- 查看实测风扇转速、ACPI 热区温度，以及系统内存和磁盘使用情况。
- 使用紧凑的 Windows 原生界面，支持系统主题、八种界面语言、关于窗口和可记住的关闭选择。见[界面说明](docs/native-ui.md)。
- 图形界面和硬件测试工具共用同一套 Rust 硬件控制代码。

**界面语言：** 简体中文、English、繁體中文、日本語、한국어、Español、Français、Deutsch。在设置中选择语言或跟随 Windows 显示语言；切换立即生效，重启后保留。

## 兼容性与硬件访问

- 访问硬件控制需要管理员权限和兼容的来酷 WMI 接口。
- 支持可选登录启动、托盘控制、单实例，以及自动/最大风量控制。热区温度尚未通过 CPU 保护验证，因此暂不开放降低风量的手动目标。
- 固件会报告实测 RPM，但不提供占空比或控制模式的读回值。详见[功能说明](docs/features.md)。
- 应用不访问 WinRing0，也不需要额外安装传感器驱动、.NET 或 VC++ 运行库。

## 下载

在 [v0.1.0 Release](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.1.0) 下载 Windows x64 版本：

- `lecoo-rust-powercontrol-v0.1.0-windows-x64.zip`：控制中心、硬件测试工具和可执行文件校验值。
- `lecoo-rust-powercontrol-v0.1.0-windows-x64-setup.exe`：安装版，包含卸载程序。

以管理员身份运行 `lecoo-control-center.exe` 才能访问硬件控制。若需自动启动，可在设置中启用“登录后启动（进入托盘）”；该选项默认关闭。验证详情和已知限制见 [v0.1.0 发布说明](docs/releases/v0.1.0.md)。

## 从源码构建

应用面向 64 位 Windows。请安装 stable Rust MSVC 工具链，以及包含 Windows SDK 的 Visual Studio Build Tools，然后运行：

```powershell
cargo fetch --locked
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-release.ps1
```

打包还需要 Inno Setup 6。脚本会在 `dist/` 生成 ZIP 与安装包；GitHub Actions 会为每个版本标签生成两种格式，并检查安装及卸载。原生 C 运行库已静态链接。

## 自动硬件测试

在兼容的 Windows 设备上运行：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\test-power-modes.ps1
```

脚本会请求管理员权限，使用独立的 Windows CIM 读取核对三种电源模式，验证非法模式值会被拒绝，并在成功或失败后恢复原模式。添加 `-ReadOnly` 可只读取硬件状态，不切换电源模式。详细 JSON 报告写入 `logs/power-mode-tests/`，该目录已从公开仓库排除。

每次分支提交和 Pull Request 都会运行 Windows CI，检查格式、Clippy、库与二进制单元测试以及发布构建；也可在 Actions 页面手动触发。覆盖范围和本地命令见[测试文档](docs/testing.md)，硬件接口细节见 [WMI 接口参考](docs/reference/wmi/interfaces.md)。

## 文档

- [界面说明](docs/native-ui.md) · [功能说明](docs/features.md) · [测试文档](docs/testing.md)
- [WMI 接口参考](docs/reference/wmi/interfaces.md) · [v0.1.0 发布说明](docs/releases/v0.1.0.md)

## 项目结构

| 路径 | 内容 |
| --- | --- |
| `src/` | Rust 应用、界面、硬件抽象和测试工具 |
| `resources/` | 八种语言各自独立的 TOML 界面资源 |
| `scripts/` | Windows 构建与自动硬件测试脚本 |
| `docs/` | 环境准备、测试和硬件接口文档 |
| `.github/` | Windows CI 与发布工作流 |

## 许可证

本项目未授予开源许可证。代码版权归 BlackSquarre 所有；仓库公开可见不代表允许转载或复用代码。
