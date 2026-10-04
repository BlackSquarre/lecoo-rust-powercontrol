# Lecoo Rust PowerControl

<p align="center">
  <strong>以 Rust 打造的輕量級 Windows 控制中心，適用於相容的來酷裝置。</strong>
</p>

<p align="center">
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases"><img src="https://img.shields.io/github/v/release/BlackSquarre/lecoo-rust-powercontrol?display_name=tag" alt="最新版本"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml/badge.svg" alt="Windows 發佈建置"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/ci.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/ci.yml/badge.svg" alt="Windows CI"></a>
  <img src="https://img.shields.io/badge/Rust-2021-orange?logo=rust" alt="Rust 2021">
  <img src="https://img.shields.io/badge/platform-Windows%20x64-blue" alt="Windows x64">
</p>

<p align="center">
  <a href="README.md">English</a> | <a href="README.zh-Hans.md">简体中文</a> | 繁體中文 | <a href="README.ja.md">日本語</a> | <a href="README.ko.md">한국어</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.de.md">Deutsch</a>
</p>

**作者：** [BlackSquarre](https://github.com/BlackSquarre) · **目前版本：** [v0.1.0](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.1.0)

> 本專案為獨立社群專案，與來酷或聯想沒有隸屬、合作或背書關係。

**目前實測裝置：** Lecoo MINI PRO-AHP · **CPU：** AMD Ryzen 7 8745H

## 功能

- 切換**安靜、均衡、效能**三種電源模式，並在切換後重新讀取硬體狀態。
- 查看實測風扇轉速、ACPI 熱區溫度，以及系統記憶體和磁碟使用情況。
- 使用精簡的 Windows 原生介面，支援系統佈景主題、八種介面語言、「關於」視窗和可記住的關閉選擇。見[介面說明](docs/native-ui.md)。
- 圖形介面和硬體測試工具共用同一套 Rust 硬體控制程式庫。

**介面語言：** 簡體中文、English、繁體中文、日本語、한국어、Español、Français、Deutsch。可在設定中選擇語言或跟隨 Windows 顯示語言；變更會立即生效並在重新啟動後保留。

## 相容性與硬體存取

- 存取硬體控制需要管理員權限和相容的來酷 WMI 提供者。
- 提供選用的登入啟動、系統匣控制、單一工作階段執行個體，以及自動/最大風量目標。熱區尚未通過 CPU 保護驗證，因此暫不開放降低風量的手動目標。
- 韌體會回報實測 RPM，但不提供風扇占空比或控制模式的讀回值。詳見[功能說明](docs/features.md)。
- 應用程式不存取 WinRing0，也不需要另外安裝感測器驅動程式、.NET 或 VC++ 執行階段。

## 下載

在 [v0.1.0 發佈頁面](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.1.0)下載 Windows x64 版本：

- `lecoo-rust-powercontrol-v0.1.0-windows-x64.zip`：應用程式、硬體測試工具和可執行檔校驗值。
- `lecoo-rust-powercontrol-v0.1.0-windows-x64-setup.exe`：含解除安裝程式的安裝版。

以管理員身分執行 `lecoo-control-center.exe` 才能存取硬體控制。若要自動啟動，可在設定中啟用「登入時啟動（最小化至系統匣）」；此選項預設為關閉。驗證細節和已知限制請見 [v0.1.0 發佈說明](docs/releases/v0.1.0.md)。

## 從原始碼建置

應用程式適用於 64 位元 Windows。請安裝 stable Rust MSVC 工具鏈，以及含 Windows SDK 的 Visual Studio Build Tools，然後執行：

```powershell
cargo fetch --locked
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-release.ps1
```

封裝還需要 Inno Setup 6。指令碼會在 `dist/` 產生 ZIP 和安裝程式；GitHub Actions 會為每個版本標籤產生兩種格式，並檢查安裝及移除流程。原生 C 執行階段已靜態連結。

## 自動硬體測試

在相容的 Windows 裝置上執行：

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\test-power-modes.ps1
```

指令碼會要求管理員權限，使用獨立的 Windows CIM 讀取來核對三種電源模式、驗證非法模式值會被拒絕，並在成功或失敗後還原原本模式。加上 `-ReadOnly` 可只讀取硬體狀態，不切換電源模式。詳細 JSON 報告會寫入 `logs/power-mode-tests/`，此目錄已從公開儲存庫排除。

每次分支推送和 Pull Request 都會執行 Windows CI，檢查格式、Clippy、程式庫與二進位檔單元測試以及發佈建置；也可在 Actions 頁面手動觸發。涵蓋範圍和本機指令請見[測試指南](docs/testing.md)，硬體介面細節請見 [WMI 介面參考](docs/reference/wmi/interfaces.md)。

## 文件

- [介面說明](docs/native-ui.md) · [功能說明](docs/features.md) · [測試指南](docs/testing.md)
- [WMI 介面參考](docs/reference/wmi/interfaces.md) · [v0.1.0 發佈說明](docs/releases/v0.1.0.md)

## 專案結構

| 路徑 | 內容 |
| --- | --- |
| `src/` | Rust 應用程式、介面、硬體抽象層和測試工具 |
| `resources/` | 八種語言各自獨立的 TOML 介面資源 |
| `scripts/` | Windows 建置和自動硬體測試指令碼 |
| `docs/` | 環境準備、測試和硬體介面文件 |
| `.github/` | Windows CI 和發佈工作流程 |

## 授權條款

本專案未授予開源授權。程式碼版權歸 BlackSquarre 所有；儲存庫公開可見不代表允許轉載或重複使用程式碼。
