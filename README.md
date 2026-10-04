# Lecoo Rust PowerControl

<p align="center">
  <strong>A Rust-powered Windows control center for compatible Lecoo systems.</strong><br>
  Switch power profiles and monitor hardware through a lightweight Windows desktop app built with Rust.
</p>

<p align="center">
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases"><img src="https://img.shields.io/github/v/release/BlackSquarre/lecoo-rust-powercontrol?display_name=tag" alt="Latest release"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml/badge.svg" alt="Windows release build"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/ci.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/ci.yml/badge.svg" alt="Windows CI"></a>
  <img src="https://img.shields.io/badge/Rust-2021-orange?logo=rust" alt="Rust 2021">
  <img src="https://img.shields.io/badge/platform-Windows%20x64-blue" alt="Windows x64">
</p>

**Author:** [BlackSquarre](https://github.com/BlackSquarre) · **Current release:** [v0.0.3](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.0.3) · **Language:** [简体中文](README.zh-CN.md)

Lecoo Rust PowerControl is an independent community project. It is not affiliated with or endorsed by Lecoo or Lenovo.

**Verified system:** Lecoo MINI PRO-AHP · **CPU:** AMD Ryzen 7 8745H

## Features

Compact native Windows controls support system themes, eight interface languages, About and a remembered close choice; see the [UI guide](docs/native-ui.md).

**Interface languages:** 简体中文 | English | 繁體中文 | 日本語 | 한국어 | Español | Français | Deutsch. Choose a language in Settings or follow the Windows display language; changes apply immediately and persist across restarts.

- Switch between **Quiet**, **Balanced**, and **Performance** power profiles.
- Read the active profile back from the hardware after a change.
- View measured fan RPM and ACPI thermal-zone temperature, plus system memory and disk usage.
- Run the GUI and hardware test utility against the same Rust hardware-control library.

Hardware access requires administrator privileges and a compatible Lecoo WMI provider. Opt-in sign-in startup, tray controls, one instance per session and Auto/Maximum fan targets are available. Reduced fan targets are disabled until the thermal zone is validated for CPU protection. No WinRing0 access or sensor driver, .NET or separate VC++ runtime is required. Firmware reports RPM, without duty or control-mode readback; see the [feature guide](docs/features.md).

## Download

Download the Windows x64 package from the [v0.0.3 release](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.0.3):

- `lecoo-rust-powercontrol-v0.0.3-windows-x64.zip` — application, hardware test utility, and executable checksums.
- `lecoo-rust-powercontrol-v0.0.3-windows-x64-setup.exe` — installer with an uninstaller.

Enable “Start at sign-in (minimized to tray)” in Settings for automatic startup; it is off by default.

Run `lecoo-control-center.exe` as administrator to access the hardware controls. See the [release notes](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.0.3) for validation details and known limitations.

## Build from source

The application targets 64-bit Windows. Install the stable Rust MSVC toolchain and Visual Studio Build Tools with the Windows SDK, then run:

```powershell
cargo fetch --locked
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-release.ps1
```

Packaging also requires Inno Setup 6. The script creates both ZIP and installer in `dist/`. GitHub Actions produces both formats for every version tag and checks installation and removal. The native C runtime is statically linked.

Every branch push and pull request runs Windows CI: formatting, Clippy, library and binary unit tests, and release builds. It can also be started manually from Actions. See the [testing guide](docs/testing.md) for coverage and local commands.

## Automated hardware test

On a compatible Windows system, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\test-power-modes.ps1
```

The script requests administrator privileges, checks all three profiles against an independent Windows CIM read, rejects an invalid mode value, and restores the original profile after success or failure. Use `-ReadOnly` to inspect hardware without changing its power profile. Detailed JSON reports are written to `logs/power-mode-tests/`, which is excluded from the public repository.

See [docs/testing.md](docs/testing.md) for parameters and recovery behavior, and [docs/reference/wmi/interfaces.md](docs/reference/wmi/interfaces.md) for the hardware interface details.

## Roadmap

Planned features are tracked in [ROADMAP.md](ROADMAP.md). Startup, tray integration, the native interface and Auto/Maximum fan control are implemented. Animation is deferred to keep resource use low.

## Project layout

```text
src/       Rust application, UI, hardware abstraction, and test utility
resources/ Independent TOML interface resources for all eight languages
scripts/   Windows build and automated hardware-test scripts
docs/      Setup, testing, and hardware-interface documentation
.github/   Windows CI and release workflows
```

## License

No open-source license is granted. Copyright remains with BlackSquarre; public access to this repository does not grant permission to redistribute or reuse the code.
