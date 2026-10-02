# Lecoo Rust PowerControl

<p align="center">
  <strong>A Rust-powered Windows control center for compatible Lecoo systems.</strong><br>
  Switch power profiles and monitor hardware through a lightweight native desktop app built with Rust and egui.
</p>

<p align="center">
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases"><img src="https://img.shields.io/github/v/release/BlackSquarre/lecoo-rust-powercontrol?display_name=tag" alt="Latest release"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml/badge.svg" alt="Windows release build"></a>
  <img src="https://img.shields.io/badge/Rust-2021-orange?logo=rust" alt="Rust 2021">
  <img src="https://img.shields.io/badge/platform-Windows%20x64-blue" alt="Windows x64">
</p>

**Author:** [BlackSquarre](https://github.com/BlackSquarre) · **Current release:** [v0.0.2](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.0.2) · **Language:** [简体中文](README.zh-CN.md)

Lecoo Rust PowerControl is an independent community project. It is not affiliated with or endorsed by Lecoo or Lenovo.

## Features

- Switch between **Quiet**, **Balanced**, and **Performance** power profiles.
- Read the active profile back from the hardware after a change.
- View measured fan RPM and native CPU Package temperature on the supported device, plus system memory and disk usage.
- Run the GUI and hardware test utility against the same Rust hardware-control library.

Hardware access requires administrator privileges and a compatible Lecoo WMI provider. The current source also includes opt-in elevated login startup, tray profile controls, one app instance per session, a close-to-tray preference, and guarded Auto/Maximum/35–100% fan requests. Native temperature and manual cooling currently require the verified 8745H single-fan path and an already loaded WinRing0 driver. The firmware cannot report actual duty or control mode; requested targets are distinguished from measured RPM. See the [feature guide](docs/features.md). Hardware switching has been verified on one Lecoo system.

## Download

Download the Windows x64 package from the [v0.0.2 release](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.0.2):

- `lecoo-rust-powercontrol-v0.0.2-windows-x64.zip` — application, hardware test utility, and executable checksums.
- `lecoo-rust-powercontrol-v0.0.2-windows-x64.zip.sha256` — checksum for the ZIP archive.

Run `lecoo-control-center.exe` as administrator to access the hardware controls. See the [release notes](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.0.2) for validation details and known limitations.

## Build from source

The application targets 64-bit Windows. Install the stable Rust MSVC toolchain and Visual Studio Build Tools with the Windows SDK, then run:

```powershell
cargo fetch --locked
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-release.ps1
```

The script builds the app and hardware test utility, copies them to `dist/`, and creates SHA-256 checksums. GitHub Actions builds release binaries from version tags using the locked dependencies.

## Automated hardware test

On a compatible Windows system, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\test-power-modes.ps1
```

The script requests administrator privileges, checks all three profiles against an independent Windows CIM read, rejects an invalid mode value, and restores the original profile after success or failure. Use `-ReadOnly` to inspect hardware without changing its power profile. Detailed JSON reports are written to `logs/power-mode-tests/`, which is excluded from the public repository.

See [docs/testing.md](docs/testing.md) for parameters and recovery behavior, and [docs/reference/wmi/interfaces.md](docs/reference/wmi/interfaces.md) for the hardware interface details.

## Roadmap

Planned features are tracked in [ROADMAP.md](ROADMAP.md). Startup, tray integration, and guarded fan control are implemented in the current source; visual redesign and animation remain planned. These additions are included in the v0.0.2 package.

## Project layout

```text
src/       Rust application, UI, hardware abstraction, and test utility
scripts/   Windows build and automated hardware-test scripts
docs/      Setup, testing, and hardware-interface documentation
.github/   Windows release workflow
```

## License

No open-source license is granted. Copyright remains with BlackSquarre; public access to this repository does not grant permission to redistribute or reuse the code.
