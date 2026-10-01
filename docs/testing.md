# 自动硬件测试

入口：`scripts/test-power-modes.ps1`。测试程序 `src/bin/hardware_test.rs` 和 GUI 共用 `src/lib.rs` 中的生产硬件控制代码。
测试要求本机 WMI 接口存在且有管理员权限；脚本可自动请求 UAC。

## 参数

| 参数 | 用途 |
| --- | --- |
| `-Cycles 1..10` | 三种模式切换的轮数，默认 1 |
| `-ReadOnly` | 只读取状态，不执行写入或非法值测试 |
| `-SkipBuild` | 使用当前 `dist` 中的发布程序 |
| `-ReportPath 路径` | 指定 JSON 报告，否则使用 `logs/power-mode-tests` 下的带时间文件名 |
| `-FailAfterFirstSwitch` | 第一次切换后故意失败，用于验证恢复；预期退出码 1 |

## 流程

1. 默认调用 `scripts/build-release.ps1` 构建并发布两个程序到 `dist`。
2. 独立 CIM 查询记录原始模式，并核对生产代码的读取结果和模式数量。
3. 普通测试拒绝非法值 3，确认模式未改变。
4. 每轮按安静(2)、均衡(0)、性能(1) 切换，分别检查生产代码和独立 CIM 的实际读回。
5. 在 finally 中恢复原始模式，独立读回验证恢复结果。生产恢复失败时尝试 CIM 恢复，整次测试仍判失败。

每次调用还读取风扇、温度、模式数量和风扇数量。不会测试手动风扇或其它未实现的设置。
不要在测试时同时用其它程序切换模式。

退出码 0 表示全部通过，1 表示失败。报告包含二进制哈希、各次调用输出、模式转换记录和恢复状态。

## 已验证结果

- [两轮共 6 次模式切换](../logs/power-mode-tests/power-modes-verified.json)：Passed=true，Restored=true。
- [故意失败后的恢复](../logs/power-mode-tests/power-modes-recovery.json)：按预期失败，从 1 切换为 2 后恢复为 1。
- [最初只读检查](../logs/power-mode-tests/power-modes-readonly.json)：Passed=true，无写入。

这些报告记录的是保存时的二进制 SHA256；后续重新编译产生的二进制应重新测试。
