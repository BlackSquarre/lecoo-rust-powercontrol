# Lecoo Rust PowerControl

<p align="center">
  <strong>Rust で構築した、Lecoo 対応システム向けの軽量な Windows コントロールセンターです。</strong>
</p>

<p align="center">
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases"><img src="https://img.shields.io/github/v/release/BlackSquarre/lecoo-rust-powercontrol?display_name=tag" alt="最新リリース"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml/badge.svg" alt="Windows リリースビルド"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/ci.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/ci.yml/badge.svg" alt="Windows CI"></a>
  <img src="https://img.shields.io/badge/Rust-2021-orange?logo=rust" alt="Rust 2021">
  <img src="https://img.shields.io/badge/platform-Windows%20x64-blue" alt="Windows x64">
</p>

<p align="center">
  <a href="README.md">English</a> | <a href="README.zh-Hans.md">简体中文</a> | <a href="README.zh-Hant.md">繁體中文</a> | 日本語 | <a href="README.ko.md">한국어</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.de.md">Deutsch</a>
</p>

**作者：** [BlackSquarre](https://github.com/BlackSquarre) · **現行リリース：** [v0.1.0](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.1.0)

> 独立したコミュニティプロジェクトです。Lecoo または Lenovo との提携・承認関係はありません。

**動作確認済みシステム：** Lecoo MINI PRO-AHP · **CPU：** AMD Ryzen 7 8745H

## 主な機能

- **静音、バランス、高性能**の 3 つの電源プロファイルを切り替え、変更後にハードウェアから状態を再取得します。
- 実測ファン回転数、ACPI 温度ゾーン、システムメモリ、ディスク使用量を確認できます。
- システムテーマ、8 言語、「このアプリについて」画面、終了方法の記憶に対応したコンパクトな Windows ネイティブ UI を備えています。[UI ガイド](docs/native-ui.md)を参照してください。
- GUI とハードウェアテストツールは、同じ Rust 製ハードウェア制御ライブラリを使用します。

**対応 UI 言語：** 简体中文、English、繁體中文、日本語、한국어、Español、Français、Deutsch。設定で言語を選択するか、Windows の表示言語に追従できます。変更はすぐに反映され、再起動後も保持されます。

## 対応環境とハードウェアアクセス

- ハードウェア制御には管理者権限と、互換性のある Lecoo WMI プロバイダーが必要です。
- サインイン時の自動起動、トレイ操作、セッションごとの単一起動、Auto/Maximum のファン目標を利用できます。CPU 保護の観点から温度ゾーンの検証が完了するまで、ファンを下げる手動目標は無効です。
- ファームウェアは実測 RPM を報告しますが、デューティ比や制御モードは読み戻せません。詳しくは[機能ガイド](docs/features.md)を参照してください。
- WinRing0 は使用せず、センサードライバー、.NET、個別の VC++ ランタイムも必要ありません。

## ダウンロード

[v0.1.0 リリース](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.1.0)から Windows x64 版をダウンロードしてください。

- `lecoo-rust-powercontrol-v0.1.0-windows-x64.zip` — アプリ、ハードウェアテストツール、実行ファイルのチェックサム。
- `lecoo-rust-powercontrol-v0.1.0-windows-x64-setup.exe` — アンインストーラー付きインストーラー。

ハードウェア制御を使うには `lecoo-control-center.exe` を管理者として実行してください。自動起動を使う場合は、設定で **Start at sign-in (minimized to tray)** を有効にします（既定ではオフ）。検証内容と既知の制限は [v0.1.0 リリースノート](docs/releases/v0.1.0.md)を参照してください。

## ソースからビルド

対象は 64 ビット Windows です。stable Rust MSVC ツールチェーンと、Windows SDK を含む Visual Studio Build Tools をインストールしてから実行します。

```powershell
cargo fetch --locked
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-release.ps1
```

パッケージ作成には Inno Setup 6 も必要です。スクリプトは `dist/` に ZIP とインストーラーを作成します。GitHub Actions は各バージョンタグで両方を生成し、インストールと削除を確認します。ネイティブ C ランタイムは静的リンクされています。

## ハードウェア自動テスト

対応する Windows システムで実行します。

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\test-power-modes.ps1
```

スクリプトは管理者権限を要求し、独立した Windows CIM 読み取りで 3 つのプロファイルを照合し、不正なモード値が拒否されることを確認します。成功時も失敗時も元のプロファイルに戻します。`-ReadOnly` を使うと電源プロファイルを変更せずに状態を確認できます。詳細な JSON レポートは公開リポジトリから除外された `logs/power-mode-tests/` に保存されます。

ブランチへの push と Pull Request ごとに Windows CI が実行され、フォーマット、Clippy、ライブラリとバイナリの単体テスト、リリースビルドを確認します。Actions から手動実行することもできます。範囲とローカルコマンドは[テストガイド](docs/testing.md)、ハードウェア詳細は [WMI インターフェイスリファレンス](docs/reference/wmi/interfaces.md)を参照してください。

## ドキュメント

- [UI ガイド](docs/native-ui.md) · [機能ガイド](docs/features.md) · [テストガイド](docs/testing.md)
- [WMI インターフェイスリファレンス](docs/reference/wmi/interfaces.md) · [v0.1.0 リリースノート](docs/releases/v0.1.0.md)

## プロジェクト構成

| パス | 内容 |
| --- | --- |
| `src/` | Rust アプリ、UI、ハードウェア抽象化、テストツール |
| `resources/` | 8 言語それぞれの独立した TOML UI リソース |
| `scripts/` | Windows ビルドおよびハードウェア自動テストスクリプト |
| `docs/` | セットアップ、テスト、ハードウェアインターフェイスの資料 |
| `.github/` | Windows CI とリリースのワークフロー |

## ライセンス

オープンソースライセンスは付与されていません。著作権は BlackSquarre に帰属します。リポジトリを公開していても、コードの再配布や再利用を許可するものではありません。
