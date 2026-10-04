# 多语言界面与关于窗口

紧凑 C 版使用 Win32 控件，语言选择与错误分类集中在 `src/localization.rs`。全部八种语言的文案和语言菜单原生名称独立保存在 `resources/locales/` 下的 UTF-8 TOML 文件中；界面及托盘仅引用统一的 `Text` 文案键，例如 `tr(Text::SettingsTitle)`。设置中的语言选择立即生效，保存在 `%LOCALAPPDATA%\LecooRustPowerControl\preferences.txt`；切换语言不会重启风扇守护进程或修改其目标。

| 菜单名称 | 偏好值 | 跟随系统时匹配的 Windows 显示语言 |
| --- | --- | --- |
| 跟随系统 | `system` | 自动选择下列语言；未支持的语言回退到英文 |
| 简体中文 | `zh-CN` | 中国大陆、新加坡及简体中文中性语言 |
| English | `en` | 英语及未支持的语言 |
| 繁體中文 | `zh-TW` | 台湾、香港、澳门及繁体中文中性语言 |
| 日本語 | `ja` | 日语 |
| 한국어 | `ko` | 韩语 |
| Español | `es` | 西班牙语，包括区域变体 |
| Français | `fr` | 法语，包括区域变体 |
| Deutsch | `de` | 德语，包括区域变体 |

语言菜单始终使用各语言的原生名称；“跟随系统”按当前语言显示。旧版 `zh-CN`、`en`、`system` 偏好及菜单前三项索引保持兼容。语言变化同时刷新主窗口、设置、关于、关闭提示、托盘菜单与模式提示。温度、RPM、GiB 单位保持一致。

`build_support/localization.rs` 在构建时解析八份资源，校验语言代码、非空文案、重复键、缺失键与多余键，再生成类型安全的文案枚举和静态字符串数组。资源内嵌于程序，运行时不解析 TOML、不读取外部翻译文件，也不增加运行时解析库。修改译文只需编辑对应资源文件，仍需重新构建；新增文案需在八份文件中补齐相同键。详见[资源维护说明](../resources/locales/README.md)。未支持的 Windows 显示语言使用英文；已支持语言缺少文案会使构建失败，未知文案键无法编译。

全部语言的用户错误提示通过资源键显示，简体中文也使用同一机制。底层中文诊断字符串仅用于内部错误分类，恢复失败、恢复请求、不可用采样等含义保持独立。

## 文案约定

| 中文 | English | 含义 |
| --- | --- | --- |
| ACPI 热区温度 | ACPI thermal zone | 固件热区读数，不代表已验证的 CPU Package 温度 |
| 风扇转速 | Fan speed | 实测转速，单位 RPM |
| 内存 / 磁盘 | Memory / Disk | 已用量及总量，容量单位保持 GiB |
| 电源模式 | Power mode | 固件电源配置 |
| 安静 / 均衡 / 性能 | Quiet / Balanced / Performance | 模式名称，分别绿、蓝、红 |
| 风扇控制 | Fan control | 固件风扇控制请求 |
| 自动 / 手动 / 最大 | Automatic / Manual / Maximum | 自动控制、手动目标、最大目标 |
| 设置 | Settings | 主窗口型号右侧的齿轮入口 |
| 登录后启动（进入托盘） | Start at sign-in (minimized to tray) | 当前用户登录后启动，默认关闭 |
| 关闭窗口时 | When closing | 设置窗口中的关闭行为标签 |
| 启动 | Startup | 设置中的登录启动分组 |
| 窗口与语言 | Window & language | 设置中的窗口行为和语言分组 |
| 应用信息 | Application | 设置底部的关于入口 |
| 每次询问 | Ask every time | 不保存固定关闭动作 |
| 最小化到托盘 / 退出 | Minimize to tray / Exit | 通知区域与真正退出是不同动作 |
| 重置关闭选择 | Reset close preference | 将已记住的动作恢复为每次询问 |
| 语言 / 跟随系统 | Language / System default | 语言设置与系统默认选项 |
| 关于… | About… | 打开关于窗口 |
| 版本 | Version | 来自 Cargo 的应用版本 |
| 哔哩哔哩 / 项目主页 | Bilibili / Project website | 默认浏览器中的外部链接 |
| 第三方声明与版权 | Third-party notices | 依赖声明与原文许可证 |
| 保留所有权利。 | All rights reserved. | 本项目的版权声明 |
| 不可用 | Unavailable | 读取失败或无有效读数 |
| 记住我的选择 | Remember my choice | 关闭对话框中的持久化选择 |

英文采用一致的句首大写风格；CPU、RPM、GiB 保持专业缩写。品牌名、作者署名和许可证原文不作推测性翻译。错误提示区分拒绝操作、不可用采样与恢复请求，不把自动恢复请求写成已确定恢复成功。

## 关于

- 名称：Lecoo Rust PowerControl；副标题：来酷迷你主机控制中心 / Lecoo mini PC control center。
- 版本由编译时 `CARGO_PKG_VERSION` 提供。
- 署名采用用户示例中的“缪凌儒 BlackSquare”。仓库地址保留实际账号拼写 BlackSquarre。
- 年份取 Windows 当前本地日历年，关于窗口打开期间也会刷新，不写死为某一年。
- 哔哩哔哩：用户指定的 `https://space.bilibili.com/404899?spm_id_from=333.1365.0.0`。
- 项目主页：`https://github.com/BlackSquarre/lecoo-rust-powercontrol`。
- 第三方声明包含默认 Windows x64 依赖图中的 63 个运行及构建依赖；从本地对应版本的原始许可证收集，33 份相同内容去重后按编号引用，保留原版权与许可证全文。历史可选图形实验依赖不在默认构建中。

关于及声明窗口都是原生窗口。查看声明、切换语言、关闭各层弹窗仍由同一消息循环处理，风扇心跳持续运行。
