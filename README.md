# Lecoo MINI PRO Control Center

An unofficial Windows control center for the Lecoo MINI PRO-AHP, written in Rust with egui.

**Author:** [BlackSquarre](https://github.com/BlackSquarre)  
**Version:** 0.0.1  
**Tested hardware:** Lecoo MINI PRO-AHP

This is an independent community project. It is not affiliated with or endorsed by Lecoo or Lenovo.

## Features

- Switch between Quiet, Balanced, and Performance modes using the local `root\WMI\PowerSwitchInterface` interface.
- Read the hardware fan and temperature values, plus system memory and disk use.
- Verify each power mode change by reading the mode back from the hardware.

Manual fan control and the settings page are not implemented yet. Hardware access requires administrator privileges. Power mode switching has been verified on one Lecoo MINI PRO-AHP.

## Download

Download `lecoo-mini-pro-ahp-control-center-v0.0.1-windows-x64.zip` from the [v0.0.1 release](https://github.com/BlackSquarre/lecoo-mini-pro-ahp-control-center/releases/tag/v0.0.1). The archive contains the control center, its hardware test utility, and SHA-256 checksums. Run the control center as administrator.

The release is built from this tag by [GitHub Actions](.github/workflows/release.yml).

## Build from source

You need the stable Rust MSVC toolchain and Visual Studio Build Tools with the Windows SDK.

```powershell
cargo fetch --locked
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-release.ps1
```

The build script compiles the app and its hardware test utility, copies both to `dist/`, and writes checksums. On a new checkout, the first build downloads locked Cargo dependencies before compiling.

## Hardware test

On a supported Lecoo MINI PRO-AHP, run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\test-power-modes.ps1
```

The script requests administrator privileges, checks all three modes against an independent Windows CIM read, and restores the original mode after the run, including its failure path. It writes detailed JSON results to `logs/power-mode-tests/`, which stays local and is excluded from the public repository.

For test options and safety details, see [docs/testing.md](docs/testing.md). WMI parameter definitions are in [docs/reference/wmi/interfaces.md](docs/reference/wmi/interfaces.md).

## License

No open-source license is granted in this release. Copyright remains with BlackSquarre; access to this public repository does not grant permission to redistribute or reuse its code.
