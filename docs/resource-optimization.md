# 资源占用优化

2026-10-04。优化前的图标和紧凑设置界面已提交并推送，基线提交为 `281e784df3eca108162ba908d90194415f2ef116`。本轮目标是减少常驻数据、后台唤醒、设备查询及短时进程开销。

## 调度与采样

| 项目 | 优化前 | 当前实现 |
| --- | --- | --- |
| 主窗口空闲定时器 | 250 ms | 1000 ms |
| 托盘自动控制空闲定时器 | 250 ms | 5000 ms |
| 风扇会话、关闭等待及滑条交互 | 250 ms | 保留 250 ms |
| 界面温度和 RPM | 每秒，隐藏后继续 | 主窗口可见时每秒 |
| 内存 | 每秒 | 主窗口可见时每两秒 |
| 磁盘 | 每秒刷新所有卷及不需要的磁盘信息 | 主窗口可见时每 30 秒查询系统盘容量 |
| 托盘模式 | 每秒 | 每五秒；应用内切换后立即读回 |
| 登录启动状态 | 启动、打开窗口及每 30 秒启动 PowerShell | 打开设置及用户操作时使用原生任务计划程序 COM |
| 系统主题 | 每两秒读注册表 | 系统变更消息及打开窗口时更新 |

空闲定时器次数由每秒四次降为窗口可见时一次、托盘时每五秒一次，分别减少 75% 和 95%。这是定时器频率变化，不是实测 CPU 或整机功耗降幅。第二次启动通过原有单实例事件唤醒；托盘操作、后台查询及守护消息通过窗口消息通知，不需要等待五秒定时器。

监视器直接持有 GUI COM apartment 内的硬件对象，移除了 `Rc<Mutex<...>>`。使用 `GlobalMemoryStatusEx` 和 `GetDiskFreeSpaceExW`，不再维护全系统及磁盘列表。系统盘路径只构造一次；读取失败显示不可用。容量查询使用一致的调用者总容量/可用容量，在磁盘配额启用时反映该用户可用的容量。

采样与渲染分离，快照以引用读取。数据和界面状态没有变化时不重新构造全部显示字符串；打开窗口、语言、主题和 DPI 变更会主动更新。控件文字复用已有字符串容量；固定文字不需要分配 `String`。关于页只在年份变化时构造版权文字。默认依赖不再包含 `sysinfo`、`rayon` 和 `tracing-subscriber`；`sysinfo` 仍作为 `legacy-memory-probe` 的可选依赖保留。

## 风扇与 WMI

风扇会话仍保留独立进程。父进程心跳间隔为 500 ms，守护检查间隔为 500 ms，心跳超时仍为三秒。收到经过恢复和 RPM 验证的自动控制响应后，GUI 请求守护正常退出，并异步检查退出结果。退出未完成时保留进程并禁止重新提交最大风量，不强制终止守护。失败退出仍走独立恢复路径，该路径复用冷却控制器的固件响应检查、RPM 验证、重试和最大风量备用请求；程序本身退出时保留原有有界等待和恢复。

守护等待下一保护期限或输入命令，移除固定 100 ms 轮询。一个保护周期只取得一份保护样本；最大风量仍检查 RPM，减少风量仍要求有效保护温度。删除 GUI 不消费的周期 `DATA` 消息及其重复 RPM/热区读取；主窗口的温度与 RPM 仍由界面自己的可见采样提供。心跳超时先请求恢复，不先执行多余的保护采样。低于最大风量的手动目标仍保持禁用。

WMI 只缓存有限的输入参数签名，每次写入都创建独立参数实例；不缓存控制结果。热区只缓存实例路径，每次读取新的对象；路径失效后重新枚举。常量属性名称使用静态 UTF-16，动态错误上下文只在失败时构造。

## 验证和测量

执行格式检查、默认目标 Clippy（警告视为错误）、全部默认目标测试及发布构建。独立测试还检查：

- 隐藏时不读取界面温度/RPM，重新打开立即读取，四类指标独立调度。
- 冷却控制的非法输入、过温、通信错误、心跳超时、恢复重试和退出恢复；每个保护周期只采样一次。
- 模拟子进程的异步关闭、成功退出及失败退出，关闭期间不向已关闭管道写心跳。
- 原生启动任务定义的用户身份、参数、触发器和权限；仅在内存中创建定义并读取现有状态，不注册或删除实际任务。
- 原生事件和窗口消息在五秒空闲定时器下仍即时唤醒。
- 四种模式、五种窗口、DPI 及十二轮窗口资源释放。

资源对照使用同一 release 配置的独立监视器测试进程，硬件返回值全部由模拟对象提供；实际系统内存/磁盘查询走生产代码。每 250 ms 执行两次监视器检查，共 15 秒，以保持优化前后的检查次数一致。它不启动实际应用、风扇守护或单实例锁。

首轮结果如下，单位 MiB：

| 监视器探针 | 15 秒结束私有提交 | 工作集 |
| --- | ---: | ---: |
| 优化前、可见采样 | 2.42 | 5.99 |
| 优化后、可见采样 | 0.86 | 4.80 |
| 优化后、隐藏采样 | 0.85 | 4.79 |

可见探针读取模式、RPM 和温度各 16 次；隐藏探针读取模式四次，RPM/温度均为零。优化前这段采样记录到 31.25 ms CPU 时间，优化后短时计数为零；计数存在分辨率限制，不能推断永久零 CPU。十二轮窗口创建/销毁后的 GDI/USER 资源均为 36/31。唤醒测试主动延迟 50 ms 发出通知，整个等待约 50–51 ms，没有等待五秒定时器。

这些是监视器与原生资源测试，不能作为完整应用、真实 WMI、整机功耗或长期无泄漏的证明。没有替换正在运行的应用；实际风扇写入和启动任务注册/删除尚未在本轮重测。

复测命令：

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --locked --offline -- -D warnings
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test-resource-usage.ps1 -Measure
cargo build --release --locked --offline --bin lecoo-control-center --bin hardware_test
cargo check --features legacy-memory-probe --bin memory_probe --locked --offline
```

原始基线输出、优化后输出、测试可执行文件哈希和测试报告位于本机忽略目录 `logs/resource-optimization/`。`verification.json` 对应首轮结果，最终验证另存为 `verification-final.json`。报告记录构建输入和测试可执行文件哈希，并检查测试期间源码是否发生变化；并发编辑时须重新编译验证。最近一轮测量保存在 `verification-measurement.json`，其中测试全部通过，但本地化构建辅助文件在测试期间发生变化，因此该轮未被标记为最终验证通过。

原生 API 语义参考微软的 [GlobalMemoryStatusEx](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-globalmemorystatusex)、[GetDiskFreeSpaceExW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getdiskfreespaceexw)、[MsgWaitForMultipleObjectsEx](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-msgwaitformultipleobjectsex) 和 [任务注册接口](https://learn.microsoft.com/en-us/windows/win32/api/taskschd/nf-taskschd-itaskfolder-registertaskdefinition)。
