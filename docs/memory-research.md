# 0.0.2 内存调查

2026-10-02 在 Lecoo MINI PRO / Ryzen 7 8745H 上测量 GitHub 实际发布的 Windows x64 程序。应用 SHA256 为 `5BDF0F892C5F1A15FD4AE3FDA0779A75B146C314338B59C29D2C10771E31A39E`。没有修改或替换已发布的 ZIP。

## 与官方程序比较

两套程序原先均未运行；同时启动，窗口可见，不切页、不进行手动调速。每约 3 秒记录进程树及 Windows 性能计数器，采样约 35 秒。下表为启动后 10–35 秒的平均值，单位 MiB。

| 程序 | 进程数 | 私有工作集 | 私有提交 |
| --- | ---: | ---: | ---: |
| Rust 0.0.2 | 1 | 123.4 | 209.7 |
| 官方主窗口进程 | 1 | 158.1 | 175.8 |
| 官方 Control Center 进程组 | 3 | 224.9 | 275.4 |

官方主窗口进程末次采样私有工作集约 158 MiB，另两个子进程各约 33 MiB。这轮未复现 Rust 的私有工作集超过官方主窗口或整个进程组；不同页面、启动时间、隐藏状态、守护进程是否启动可能改变比较结果。

“私有工作集”是当前驻留内存的私有页；“私有提交”包含尚未驻留的已提交页；“工作集”还包含共享页。不能混用这些数字，也不能把进程组与其中一个子进程比较。多个进程的总工作集可能重复计算共享页，因此本次主要比较私有工作集。

Rust 私有工作集从启动约 136 MiB 下降到后期约 121 MiB，私有提交从约 241 MiB 下降到约 207 MiB。这段短时采样没有持续上涨，不是长期无泄漏的证明。测试结束只关闭测试新开的进程，恢复自动风扇控制，并独立确认电源模式恢复为测试前的 1。

## 分项原因

独立诊断入口 [`memory_probe`](../src/bin/memory_probe.rs) 在不同子进程分别运行，直接用 Windows `GetProcessMemoryInfo` 测量；它不调用硬件控制。下列实验是各自进程的结果，不可简单相加为应用总量。

1. **全系统信息初始化读取了用不到的内容。** `SystemMonitor::new` 使用 `System::new_all()`，除了内存，还读取并保留 250 个进程和 16 个逻辑 CPU 的信息；界面实际上只读取内存总量/用量。完整初始化使私有提交增加约 9.0 MiB，仅创建 `System::new()` 并刷新内存约增加 0.8 MiB，差约 8.2 MiB。
2. **字体数据保留两份。** 主题读取整个 `msyh.ttc`，文件为 19,704,352 字节，即 18.8 MiB。`FontData::from_owned` 保留它，epaint 0.30 的 `ab_glyph_font_from_font_data` 对 Owned 数据执行 `bytes.clone()`。字体渲染后私有提交较空 context 增加约 39.1 MiB。借用同一字体数据的对照实验为约 20.2 MiB，确认额外复制约 18.9 MiB。此处不是把系统字体误算为小小的几百 KB。
3. **图形框架和 OpenGL 上下文存在较大基线。** 不读取系统信息、不加载中文字体，只创建同为 1000×600 的 eframe 默认图形窗口，工作集约 114.3 MiB，私有提交约 180.1 MiB。这个实验包含窗口系统、图形后端、驱动、默认字体和相关缓存，不能全部归因于 Rust 对象或某个驱动模块。

`font-borrowed` 只用于验证复制成本，实验以进程寿命存储保持借用有效；该实验实现没有接入产品。生产优化应使用寿命受控的共享/只读映射，不能为了少分配而制造悬空字体数据。

优先优化无用的系统扫描与字体副本，再评估图形后端；后端替换需要同时比较内存、CPU、渲染正确性和硬件兼容性。本次完成调查，不改变 0.0.2 的已发布二进制或图形后端。

## 复测

```powershell
cargo build --release --locked --bin memory_probe
# 在管理员终端逐个运行；graphics 会短暂显示一个诊断窗口。
.\target\release\memory_probe.exe system-full
.\target\release\memory_probe.exe system-memory
.\target\release\memory_probe.exe font-owned
.\target\release\memory_probe.exe font-borrowed
.\target\release\memory_probe.exe graphics
```

原始进程树与分项输出分别保存在本机忽略目录 `logs/memory-research/comparison-v0.0.2.json` 和 `components.json`。进程对照包含官方程序的子进程；没有只挑其中较小的辅助进程作为官方程序用量。
