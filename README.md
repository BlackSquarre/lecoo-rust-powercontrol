# Lecoo Rust PowerControl

<p align="center">
  <strong>A lightweight Windows control center for compatible Lecoo systems, built with Rust.</strong>
</p>

<p align="center">
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases"><img src="https://img.shields.io/github/v/release/BlackSquarre/lecoo-rust-powercontrol?display_name=tag" alt="Latest release"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml/badge.svg" alt="Windows release build"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/ci.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/ci.yml/badge.svg" alt="Windows CI"></a>
  <img src="https://img.shields.io/badge/Rust-2021-orange?logo=rust" alt="Rust 2021">
  <img src="https://img.shields.io/badge/platform-Windows%20x64-blue" alt="Windows x64">
</p>

<p align="center">
  English | <a href="README.zh-Hans.md">简体中文</a> | <a href="README.zh-Hant.md">繁體中文</a> | <a href="README.ja.md">日本語</a> | <a href="README.ko.md">한국어</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.de.md">Deutsch</a>
</p>

**Author:** [BlackSquarre](https://github.com/BlackSquarre) · **Current release:** [v0.1.0](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.1.0)

> Independent community project. Not affiliated with or endorsed by Lecoo or Lenovo.

**Verified system:** Lecoo MINI PRO-AHP · **CPU:** AMD Ryzen 7 8745H

## Features

- Switch between **Quiet**, **Balanced**, and **Performance** power profiles, then read the active profile back from the hardware.
- Monitor measured fan speed, ACPI thermal-zone temperature, system memory, and disk usage.
- Use a compact native Windows interface with system themes, eight interface languages, an About window, and a remembered close choice. See the [UI guide](docs/native-ui.md).
- Run the GUI and hardware test utility against the same Rust hardware-control library.

**Interface languages:** 简体中文, English, 繁體中文, 日本語, 한국어, Español, Français, and Deutsch. Choose a language in Settings or follow the Windows display language. Changes apply immediately and persist across restarts.

## Compatibility and hardware access

- Hardware controls require administrator privileges and a compatible Lecoo WMI provider.
- Optional sign-in startup, tray controls, one instance per session, and Auto/Maximum fan targets are available. Reduced fan targets remain disabled until the thermal zone is validated for CPU protection.
- Firmware reports measured RPM, but does not provide duty or control-mode readback. See the [feature guide](docs/features.md).
- No WinRing0 access or sensor driver, .NET, or separate VC++ runtime is required.

## Download

Download the Windows x64 package from the [v0.1.0 release](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.1.0):

- `lecoo-rust-powercontrol-v0.1.0-windows-x64.zip` — application, hardware test utility, and executable checksums.
- `lecoo-rust-powercontrol-v0.1.0-windows-x64-setup.exe` — installer with an uninstaller.

Run `lecoo-control-center.exe` as administrator to access hardware controls. To start the app automatically, enable **Start at sign-in (minimized to tray)** in Settings; this option is off by default. See the [release notes](docs/releases/v0.1.0.md) for validation details and known limitations.

## Build from source

The application targets 64-bit Windows. Install the stable Rust MSVC toolchain and Visual Studio Build Tools with the Windows SDK, then run:

```powershell
cargo fetch --locked
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-release.ps1
```

Packaging also requires Inno Setup 6. The script creates a ZIP and installer in `dist/`. GitHub Actions produces both formats for every version tag and checks installation and removal. The native C runtime is statically linked.

## Automated hardware test

On a compatible Windows system, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\test-power-modes.ps1
```

The script requests administrator privileges, checks all three profiles against an independent Windows CIM read, rejects an invalid mode value, and restores the original profile after success or failure. Use `-ReadOnly` to inspect hardware without changing its power profile. Detailed JSON reports are written to `logs/power-mode-tests/`, which is excluded from the public repository.

Every branch push and pull request runs Windows CI for formatting, Clippy, library and binary unit tests, and release builds. CI can also be started manually from Actions. See the [testing guide](docs/testing.md) for coverage and local commands, and the [WMI interface reference](docs/reference/wmi/interfaces.md) for hardware details.

## Documentation

- [UI guide](docs/native-ui.md) · [Feature guide](docs/features.md) · [Testing guide](docs/testing.md)
- [WMI interface reference](docs/reference/wmi/interfaces.md) · [v0.1.0 release notes](docs/releases/v0.1.0.md)

## Project layout

| Path | Contents |
| --- | --- |
| `src/` | Rust application, UI, hardware abstraction, and test utility |
| `resources/` | Independent TOML interface resources for all eight languages |
| `scripts/` | Windows build and automated hardware-test scripts |
| `docs/` | Setup, testing, and hardware-interface documentation |
| `.github/` | Windows CI and release workflows |

## License

No open-source license is granted. Copyright remains with BlackSquarre; public access to this repository does not grant permission to redistribute or reuse the code.
