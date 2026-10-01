# Lecoo MINI PRO 控制中心

基于 Rust 和 egui 为 Lecoo MINI PRO-AHP 开发的非官方 Windows 控制中心。

**作者：** [BlackSquarre](https://github.com/BlackSquarre)  
**版本：** 0.0.1  
**已测试设备：** Lecoo MINI PRO-AHP

这是独立社区项目，与 Lecoo 或 Lenovo 无隶属、合作或背书关系。

## 功能

- 通过本机 `root\WMI\PowerSwitchInterface` 接口切换安静、均衡和性能模式。
- 读取风扇与温度数据，以及系统内存和磁盘使用情况。
- 切换模式后重新读取硬件状态，验证切换结果。

风扇手动控制和设置页面尚未实现。访问硬件需要管理员权限。电源模式切换已在一台 Lecoo MINI PRO-AHP 上验证。

## 下载

在 [v0.0.1 Release](https://github.com/BlackSquarre/lecoo-mini-pro-ahp-control-center/releases/tag/v0.0.1) 下载 `lecoo-mini-pro-ahp-control-center-v0.0.1-windows-x64.zip`。压缩包内含控制中心、硬件测试工具和 SHA-256 校验文件。请以管理员身份运行控制中心。

GitHub Actions 根据 [v0.0.1 标签](.github/workflows/release.yml) 构建发布包。

## 从源码构建

需要 stable Rust MSVC 工具链，以及安装了 Windows SDK 的 Visual Studio Build Tools。

```powershell
cargo fetch --locked
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-release.ps1
```

构建脚本会编译 App 和硬件测试工具，将其复制到 `dist/` 并生成校验文件。首次克隆后，先下载 Cargo 锁定的依赖再构建。

## 硬件测试

在受支持的 Lecoo MINI PRO-AHP 上运行：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\test-power-modes.ps1
```

脚本会请求管理员权限，通过独立的 Windows CIM 读取核对三种模式，并在测试正常结束或失败时恢复原模式。详细 JSON 写入本机的 `logs/power-mode-tests/`；这些日志已从公开仓库中排除。

参数说明和恢复行为见[自动测试文档](docs/testing.md)，WMI 参数定义见[接口文档](docs/reference/wmi/interfaces.md)。

## 许可证

此版本未授予开源许可证。代码版权归 BlackSquarre 所有；访问公开仓库不代表获得转载或复用许可。
