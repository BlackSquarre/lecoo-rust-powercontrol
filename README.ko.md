# Lecoo Rust PowerControl

<p align="center">
  <strong>호환되는 Lecoo 시스템을 위한 Rust 기반의 가벼운 Windows 제어 센터입니다.</strong>
</p>

<p align="center">
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases"><img src="https://img.shields.io/github/v/release/BlackSquarre/lecoo-rust-powercontrol?display_name=tag" alt="최신 릴리스"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml/badge.svg" alt="Windows 릴리스 빌드"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/ci.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/ci.yml/badge.svg" alt="Windows CI"></a>
  <img src="https://img.shields.io/badge/Rust-2021-orange?logo=rust" alt="Rust 2021">
  <img src="https://img.shields.io/badge/platform-Windows%20x64-blue" alt="Windows x64">
</p>

<p align="center">
  <a href="README.md">English</a> | <a href="README.zh-Hans.md">简体中文</a> | <a href="README.zh-Hant.md">繁體中文</a> | <a href="README.ja.md">日本語</a> | 한국어 | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | <a href="README.de.md">Deutsch</a>
</p>

**작성자:** [BlackSquarre](https://github.com/BlackSquarre) · **현재 릴리스:** [v0.1.0](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.1.0)

> 독립 커뮤니티 프로젝트입니다. Lecoo 또는 Lenovo와 제휴하거나 보증을 받은 제품이 아닙니다.

**검증된 시스템:** Lecoo MINI PRO-AHP · **CPU:** AMD Ryzen 7 8745H

## 기능

- **저소음, 균형, 성능** 전원 프로필을 전환하고, 변경 후 하드웨어에서 활성 프로필을 다시 읽습니다.
- 실제 팬 회전 속도, ACPI 온도 영역, 시스템 메모리 및 디스크 사용량을 확인합니다.
- 시스템 테마, 8개 인터페이스 언어, 정보 창, 기억된 종료 선택을 지원하는 간결한 Windows 네이티브 UI를 제공합니다. [UI 가이드](docs/native-ui.md)를 참조하세요.
- GUI와 하드웨어 테스트 도구가 동일한 Rust 하드웨어 제어 라이브러리를 사용합니다.

**인터페이스 언어:** 简体中文, English, 繁體中文, 日本語, 한국어, Español, Français, Deutsch. 설정에서 언어를 선택하거나 Windows 표시 언어를 따를 수 있습니다. 변경 사항은 즉시 적용되며 재시작 후에도 유지됩니다.

## 호환성 및 하드웨어 접근

- 하드웨어 제어에는 관리자 권한과 호환되는 Lecoo WMI 공급자가 필요합니다.
- 로그인 시 자동 시작, 트레이 제어, 세션당 단일 인스턴스, Auto/Maximum 팬 목표를 사용할 수 있습니다. CPU 보호를 위해 온도 영역 검증이 완료될 때까지 팬 속도를 낮추는 수동 목표는 비활성화되어 있습니다.
- 펌웨어는 실제 RPM을 보고하지만 듀티나 제어 모드 값을 다시 읽을 수는 없습니다. 자세한 내용은 [기능 가이드](docs/features.md)를 참조하세요.
- WinRing0를 사용하지 않으며 별도의 센서 드라이버, .NET 또는 VC++ 런타임이 필요하지 않습니다.

## 다운로드

[v0.1.0 릴리스](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.1.0)에서 Windows x64 패키지를 다운로드하세요.

- `lecoo-rust-powercontrol-v0.1.0-windows-x64.zip` — 앱, 하드웨어 테스트 도구, 실행 파일 체크섬.
- `lecoo-rust-powercontrol-v0.1.0-windows-x64-setup.exe` — 제거 프로그램이 포함된 설치 파일.

하드웨어 제어를 사용하려면 `lecoo-control-center.exe`를 관리자 권한으로 실행하세요. 자동 시작을 원하면 설정에서 **Start at sign-in (minimized to tray)**를 활성화하세요. 기본값은 꺼짐입니다. 검증 세부 정보와 알려진 제한은 [v0.1.0 릴리스 노트](docs/releases/v0.1.0.md)를 참조하세요.

## 소스에서 빌드

이 앱은 64비트 Windows를 대상으로 합니다. stable Rust MSVC 도구 체인과 Windows SDK가 포함된 Visual Studio Build Tools를 설치한 뒤 다음을 실행하세요.

```powershell
cargo fetch --locked
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-release.ps1
```

패키징에는 Inno Setup 6도 필요합니다. 스크립트는 `dist/`에 ZIP과 설치 파일을 만듭니다. GitHub Actions는 각 버전 태그마다 두 형식을 만들고 설치 및 제거를 확인합니다. 네이티브 C 런타임은 정적으로 링크되어 있습니다.

## 자동 하드웨어 테스트

호환되는 Windows 시스템에서 실행하세요.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\test-power-modes.ps1
```

스크립트는 관리자 권한을 요청하고 독립적인 Windows CIM 읽기로 세 가지 프로필을 확인하며, 잘못된 모드 값이 거부되는지 검증합니다. 성공하거나 실패한 뒤 원래 프로필을 복원합니다. `-ReadOnly`를 사용하면 전원 프로필을 변경하지 않고 하드웨어 상태를 확인할 수 있습니다. 상세 JSON 보고서는 공개 저장소에서 제외된 `logs/power-mode-tests/`에 저장됩니다.

브랜치 푸시와 Pull Request마다 Windows CI가 포맷, Clippy, 라이브러리 및 바이너리 단위 테스트, 릴리스 빌드를 실행합니다. Actions에서 수동으로 시작할 수도 있습니다. 범위와 로컬 명령은 [테스트 가이드](docs/testing.md), 하드웨어 세부 정보는 [WMI 인터페이스 참고](docs/reference/wmi/interfaces.md)를 참조하세요.

## 문서

- [UI 가이드](docs/native-ui.md) · [기능 가이드](docs/features.md) · [테스트 가이드](docs/testing.md)
- [WMI 인터페이스 참고](docs/reference/wmi/interfaces.md) · [v0.1.0 릴리스 노트](docs/releases/v0.1.0.md)

## 프로젝트 구조

| 경로 | 내용 |
| --- | --- |
| `src/` | Rust 앱, UI, 하드웨어 추상화, 테스트 도구 |
| `resources/` | 8개 언어별 독립 TOML UI 리소스 |
| `scripts/` | Windows 빌드 및 자동 하드웨어 테스트 스크립트 |
| `docs/` | 설정, 테스트, 하드웨어 인터페이스 문서 |
| `.github/` | Windows CI 및 릴리스 워크플로 |

## 라이선스

오픈 소스 라이선스가 부여되지 않았습니다. 저작권은 BlackSquarre에 있으며 저장소가 공개되어 있어도 코드 재배포나 재사용을 허용하는 것은 아닙니다.
