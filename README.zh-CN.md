# Lecoo Rust PowerControl

<p align="center">
  <strong>使用 Rust 构建的来酷 Windows 控制中心。</strong><br>
  通过轻量级原生桌面应用切换电源模式并查看硬件状态，界面使用 Rust 和 egui 实现。
</p>

<p align="center">
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases"><img src="https://img.shields.io/github/v/release/BlackSquarre/lecoo-rust-powercontrol?display_name=tag" alt="最新版本"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml/badge.svg" alt="Windows 发布构建"></a>
  <img src="https://img.shields.io/badge/Rust-2021-orange?logo=rust" alt="Rust 2021">
  <img src="https://img.shields.io/badge/platform-Windows%20x64-blue" alt="Windows x64">
</p>

**作者：** [BlackSquarre](https://github.com/BlackSquarre) · **当前版本：** [v0.0.1](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.0.1) · **Language:** [English](README.md)

Lecoo Rust PowerControl 是独立社区项目，与来酷或联想没有隶属、合作或背书关系。

## 功能

- 切换**安静、均衡、性能**三种电源模式。
- 切换后重新读取硬件状态，确认模式是否生效。
- 查看风扇转速和固件支持时的 CPU 温度，以及系统内存、磁盘使用情况。
- 图形界面和硬件测试工具共用同一套 Rust 硬件控制代码。

访问硬件需要管理员权限和兼容的来酷 WMI 接口。手动风扇调速和设置页操作尚未实现。电源模式切换已在一台来酷设备上完成硬件验证。

## 下载

在 [v0.0.1 Release](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.0.1) 下载 Windows x64 版本：

- `lecoo-rust-powercontrol-v0.0.1-windows-x64.zip`：控制中心、硬件测试工具和可执行文件校验值。
- `lecoo-rust-powercontrol-v0.0.1-windows-x64.zip.sha256`：压缩包校验值。

以管理员身份运行 `lecoo-control-center.exe` 才能访问硬件控制。验证详情和已知限制见 [Release 日志](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.0.1)。

## 从源码构建

应用面向 64 位 Windows。请安装 stable Rust MSVC 工具链，以及包含 Windows SDK 的 Visual Studio Build Tools，然后运行：

```powershell
cargo fetch --locked
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-release.ps1
```

脚本会构建图形应用和硬件测试工具，将文件复制到 `dist/` 并生成 SHA-256 校验值。GitHub Actions 使用锁定的依赖，从版本标签构建发布程序。

## 自动硬件测试

在兼容的 Windows 设备上运行：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\test-power-modes.ps1
```

脚本会请求管理员权限，使用独立的 Windows CIM 读取核对三种电源模式，验证非法模式值会被拒绝，并在成功或失败后恢复原模式。添加 `-ReadOnly` 可只读取硬件状态，不切换电源模式。详细 JSON 报告写入 `logs/power-mode-tests/`；该目录已从公开仓库排除。

参数和恢复行为见[测试文档](docs/testing.md)，硬件接口说明见 [WMI 接口文档](docs/reference/wmi/interfaces.md)。

## 后续计划

后续功能规划见[路线图](ROADMAP.md)。其中列出的项目都还没有实现。

## 项目结构

```text
src/       Rust 应用、界面、硬件抽象和测试工具
scripts/   Windows 构建与自动硬件测试脚本
docs/      环境准备、测试和硬件接口文档
.github/   Windows 发布工作流
```

## 许可证

本项目未授予开源许可证。代码版权归 BlackSquarre 所有；仓库公开可见不代表允许转载或复用代码。
