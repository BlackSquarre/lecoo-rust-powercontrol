# Visual Studio Build Tools 安装指南

**阶段**: 第三阶段 - 编译环境准备  
**日期**: 2026-10-02  
**目标**: 安装 Microsoft Visual Studio Build Tools 以支持 Rust MSVC 工具链

---

## 📋 问题诊断

### 当前环境问题

之前编译失败的原因：

```bash
$ which link.exe
/usr/bin/link.exe

$ link.exe --version
link (GNU coreutils) 8.32
```

**问题**: 系统 PATH 中的 `link.exe` 是 GNU coreutils 的链接工具，不是 Microsoft 链接器。

Rust MSVC 工具链需要：
- `link.exe` (Microsoft Linker)
- `lib.exe` (Microsoft Librarian)
- Windows SDK 头文件和库

---

## 🎯 解决方案

### 选项 1: 安装 Visual Studio Build Tools（推荐）

**优势**:
- ✅ 官方支持
- ✅ 完整的 Windows 开发环境
- ✅ 自动配置环境变量
- ✅ 支持未来扩展（C++ 互操作）

**劣势**:
- ⚠️ 下载较大（~6-8 GB）
- ⚠️ 安装时间较长（20-40 分钟）

---

## 📥 安装步骤

### 步骤 1: 下载安装器

**官方下载页面**: https://visualstudio.microsoft.com/downloads/

1. 滚动到页面底部 "所有下载"
2. 展开 "Tools for Visual Studio"
3. 下载 **Build Tools for Visual Studio 2022**

**直接下载链接**:
```
https://aka.ms/vs/17/release/vs_BuildTools.exe
```

### 步骤 2: 运行安装器

1. 以管理员身份运行 `vs_BuildTools.exe`
2. 等待安装器启动

### 步骤 3: 选择工作负载

在安装器界面中，选择以下组件：

#### ✅ 必选项

**工作负载**:
- ☑️ **Desktop development with C++** (使用 C++ 的桌面开发)

**单个组件**（在 "Individual components" 标签页）:
- ☑️ MSVC v143 - VS 2022 C++ x64/x86 build tools (latest)
- ☑️ Windows 11 SDK (10.0.22621.0 或更新)
- ☑️ C++ CMake tools for Windows

#### ⬜ 可选项

如果磁盘空间充足，可额外选择：
- ⬜ C++ Clang tools for Windows
- ⬜ C++ AddressSanitizer

### 步骤 4: 确认安装位置

**默认路径**:
```
C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools
```

**所需空间**: 约 8-10 GB

### 步骤 5: 开始安装

1. 点击 "Install" 按钮
2. 等待下载和安装完成（20-40 分钟）
3. 安装完成后点击 "Close"

---

## ✅ 验证安装

### 方法 1: 检查链接器路径

打开 **x64 Native Tools Command Prompt for VS 2022**:

```cmd
where link.exe
```

**预期输出**:
```
C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.xx.xxxxx\bin\Hostx64\x64\link.exe
```

### 方法 2: 检查链接器版本

```cmd
link.exe
```

**预期输出**:
```
Microsoft (R) Incremental Linker Version 14.xx.xxxxx
Copyright (C) Microsoft Corporation.  All rights reserved.

usage: LINK [options] [files] [@commandfile]
...
```

### 方法 3: 从 Claude Code 验证

```bash
# 列出 VS 安装路径
ls -la "C:/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools/VC/Tools/MSVC/"

# 检查环境变量（需要重启 Claude Code）
echo $PATH | tr ':' '\n' | grep -i msvc
```

---

## 🔧 配置 Rust 工具链

### 步骤 1: 切换到 MSVC 工具链

```bash
rustup default stable-x86_64-pc-windows-msvc
```

**预期输出**:
```
info: using existing install for 'stable-x86_64-pc-windows-msvc'
info: default toolchain set to 'stable-x86_64-pc-windows-msvc'
```

### 步骤 2: 验证工具链

```bash
rustc --version --verbose
```

**预期输出**:
```
rustc 1.xx.x (xxxxxxx 2024-xx-xx)
binary: rustc
commit-hash: ...
commit-date: ...
host: x86_64-pc-windows-msvc        # ← 确认是 msvc
release: 1.xx.x
LLVM version: xx.x.x
```

### 步骤 3: 清理之前的构建产物

```bash
cd <path-to-project>
cargo clean
```

### 步骤 4: 测试编译

```bash
cargo build --release
```

**预期**: 成功编译，无链接器错误

---

## 🚨 常见问题

### 问题 1: 安装器无法启动

**解决方案**:
1. 确保以管理员身份运行
2. 检查 Windows Update 是否有待处理的更新
3. 临时关闭杀毒软件

### 问题 2: 磁盘空间不足

**解决方案**:
```powershell
# 检查可用空间
Get-PSDrive C | Select-Object Used,Free

# 如果空间不足，清理临时文件
Disk Cleanup
```

**最小化安装**: 只选择 "MSVC build tools" 和 "Windows SDK"，可减少到约 4-5 GB

### 问题 3: Rust 仍然找不到链接器

**原因**: Rust 需要通过特定的环境变量找到 MSVC

**解决方案 A**: 使用 VS Developer Command Prompt

1. 打开 "x64 Native Tools Command Prompt for VS 2022"
2. 在该终端中运行 `cargo build`

**解决方案 B**: 配置 `.cargo/config.toml`

创建文件 `.cargo/config.toml`:

```toml
[target.x86_64-pc-windows-msvc]
linker = "C:\\Program Files (x86)\\Microsoft Visual Studio\\2022\\BuildTools\\VC\\Tools\\MSVC\\14.xx.xxxxx\\bin\\Hostx64\\x64\\link.exe"

[build]
target = "x86_64-pc-windows-msvc"
```

**注意**: 将 `14.xx.xxxxx` 替换为实际安装的版本号

### 问题 4: Claude Code 重启后仍找不到

**原因**: Claude Code 会话可能缓存了旧的环境变量

**解决方案**:
1. 完全关闭 Claude Code
2. 重新以管理员身份启动
3. 验证环境变量：
   ```bash
   echo $PATH | tr ':' '\n' | grep -i Microsoft
   ```

---

## 🔍 替代方案

### 选项 2: 使用 MinGW (GNU 工具链)

如果无法安装 Visual Studio Build Tools，可以尝试：

```bash
# 切换到 GNU 工具链
rustup default stable-x86_64-pc-windows-gnu

# 安装 MinGW-w64
# 但需要 pacman 或其他包管理器，当前环境可能不支持
```

⚠️ **限制**:
- `windows-rs` crate 在 GNU 工具链上可能有兼容性问题
- 部分 COM 功能可能不可用
- 不推荐用于 Windows API 密集型项目

---

## 📝 安装清单

### 安装前

- [ ] 检查可用磁盘空间（至少 10 GB）
- [ ] 确保有管理员权限
- [ ] 备份当前项目（可选）

### 安装中

- [ ] 下载 `vs_BuildTools.exe`
- [ ] 以管理员身份运行安装器
- [ ] 选择 "Desktop development with C++" 工作负载
- [ ] 确认包含 MSVC 和 Windows SDK
- [ ] 开始安装并等待完成

### 安装后

- [ ] 验证 `link.exe` 是 Microsoft 版本
- [ ] 切换 Rust 到 MSVC 工具链
- [ ] 运行 `cargo clean`
- [ ] 测试编译项目
- [ ] 重启 Claude Code（如果需要）

---

## 🎯 预期结果

安装成功后，你应该能够：

1. ✅ 运行 `cargo build` 无链接器错误
2. ✅ 编译包含 `windows-rs` 的项目
3. ✅ 生成可执行文件 `target/release/lecoo-control-center.exe`
4. ✅ 直接运行生成的 EXE 文件

---

## 📚 参考资源

- [Visual Studio Build Tools 官方页面](https://visualstudio.microsoft.com/downloads/)
- [Rust 官方 Windows 安装指南](https://rust-lang.github.io/rustup/installation/windows.html)
- [windows-rs 文档](https://microsoft.github.io/windows-docs-rs/)

---

## ✅ 完成确认

- [ ] Visual Studio Build Tools 已安装
- [ ] Microsoft `link.exe` 可用
- [ ] Rust MSVC 工具链已配置
- [ ] 测试编译成功

---

## 🎯 下一步

安装完成后，进入 **第四阶段：Rust 代码实现**

文档文件: [docs/04-implementation-phase1.md](../../README.md)


