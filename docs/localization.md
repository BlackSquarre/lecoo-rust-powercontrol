# 中英文界面与关于窗口

紧凑 C 版使用 Win32 控件，语言资源集中在 `src/localization.rs`，文案配对在界面和托盘构造处。默认跟随 Windows 显示语言：中文系统使用简体中文，其余系统使用英文。设置中的语言选择可立即覆盖默认，保存在 `%LOCALAPPDATA%\LecooRustPowerControl\preferences.txt`；切换语言不会重启风扇守护进程或修改其目标。

## 文案约定

| 中文 | English | 含义 |
| --- | --- | --- |
| CPU 温度 | CPU temperature | AMD Tctl/Tdie 温度源 |
| CPU Package 功率 | CPU package power | 采样区间的封装平均功率，单位 W |
| 风扇转速 | Fan speed | 实测转速，单位 RPM |
| 内存 / 磁盘 | Memory / Disk | 已用量及总量，容量单位保持 GiB |
| 电源模式 | Power mode | 固件电源配置 |
| 安静 / 均衡 / 性能 | Quiet / Balanced / Performance | 模式名称，分别绿、蓝、红 |
| 风扇控制 | Fan control | 固件风扇控制请求 |
| 自动 / 手动 / 最大 | Automatic / Manual / Maximum | 自动控制、手动目标、最大目标 |
| 设置 | Settings | 主窗口型号右侧的齿轮入口 |
| 登录后启动（进入托盘） | Start at sign-in (minimized to tray) | 当前用户登录后启动，默认关闭 |
| 关闭窗口 | When closing the window | 设置窗口中的关闭行为标签 |
| 每次询问 | Ask every time | 不保存固定关闭动作 |
| 最小化到托盘 / 退出 | Minimize to tray / Exit | 通知区域与真正退出是不同动作 |
| 重置关闭选择 | Reset close preference | 将已记住的动作恢复为每次询问 |
| 语言 / 跟随系统 | Language / System default | 语言设置与系统默认选项 |
| 关于… | About… | 打开关于窗口 |
| 版本 | Version | 来自 Cargo 的应用版本 |
| 哔哩哔哩 / 项目主页 | Bilibili / Project website | 默认浏览器中的外部链接 |
| 第三方声明与版权 | Third-party notices | 依赖声明与原文许可证 |
| 保留所有权利。 | All rights reserved. | 本项目的版权声明 |
| 不可用 | Unavailable | 读取失败或尚未形成有效功率采样 |
| 记住我的选择 | Remember my choice | 关闭对话框中的持久化选择 |

英文采用一致的句首大写风格；CPU、RPM、W、GiB 保持专业缩写。品牌名、作者署名和许可证原文不作推测性翻译。错误提示区分拒绝操作、不可用采样与恢复请求，不把自动恢复请求写成已确定恢复成功。

## 关于

- 名称：Lecoo Rust PowerControl；副标题：来酷迷你主机控制中心 / Lecoo mini PC control center。
- 版本由编译时 `CARGO_PKG_VERSION` 提供。
- 署名采用用户示例中的“缪凌儒 BlackSquare”。仓库地址保留实际账号拼写 BlackSquarre。
- 年份取 Windows 当前本地日历年，关于窗口打开期间也会刷新，不写死为某一年。
- 哔哩哔哩：用户指定的 `https://space.bilibili.com/404899?spm_id_from=333.1365.0.0`。
- 项目主页：`https://github.com/BlackSquarre/lecoo-rust-powercontrol`。
- 第三方声明包含默认 Windows x64 依赖图中的 63 个运行及构建依赖；从本地对应版本的原始许可证收集，33 份相同内容去重后按编号引用，保留原版权与许可证全文。历史可选图形实验依赖不在默认构建中。

关于及声明窗口都是原生窗口。查看声明、切换语言、关闭各层弹窗仍由同一消息循环处理，风扇心跳持续运行。
