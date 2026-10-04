# Lecoo Rust PowerControl

<p align="center">
  <strong>Ein schlankes Windows-Kontrollzentrum auf Rust-Basis für kompatible Lecoo-Systeme.</strong>
</p>

<p align="center">
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases"><img src="https://img.shields.io/github/v/release/BlackSquarre/lecoo-rust-powercontrol?display_name=tag" alt="Neueste Version"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml/badge.svg" alt="Windows-Release-Build"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/ci.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/ci.yml/badge.svg" alt="Windows-CI"></a>
  <img src="https://img.shields.io/badge/Rust-2021-orange?logo=rust" alt="Rust 2021">
  <img src="https://img.shields.io/badge/platform-Windows%20x64-blue" alt="Windows x64">
</p>

<p align="center">
  <a href="README.md">English</a> | <a href="README.zh-Hans.md">简体中文</a> | <a href="README.zh-Hant.md">繁體中文</a> | <a href="README.ja.md">日本語</a> | <a href="README.ko.md">한국어</a> | <a href="README.es.md">Español</a> | <a href="README.fr.md">Français</a> | Deutsch
</p>

**Autor:** [BlackSquarre](https://github.com/BlackSquarre) · **Aktuelle Version:** [v0.1.0](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.1.0)

> Unabhängiges Community-Projekt. Es besteht keine Zugehörigkeit oder Empfehlung durch Lecoo oder Lenovo.

**Verifiziertes System:** Lecoo MINI PRO-AHP · **CPU:** AMD Ryzen 7 8745H

## Funktionen

- Wechsel zwischen den Energieprofilen **Leise, Ausgewogen und Leistung**; anschließend wird das aktive Profil erneut von der Hardware ausgelesen.
- Anzeige der gemessenen Lüfterdrehzahl, der ACPI-Temperaturzone sowie der Speicher- und Datenträgerauslastung.
- Kompakte native Windows-Oberfläche mit Systemdesigns, acht Oberflächensprachen, Infofenster und speicherbarer Schließauswahl. Siehe den [UI-Leitfaden](docs/native-ui.md).
- GUI und Hardwaretestprogramm verwenden dieselbe Rust-Bibliothek zur Hardwaresteuerung.

**Oberflächensprachen:** 简体中文, English, 繁體中文, 日本語, 한국어, Español, Français und Deutsch. Wähle die Sprache in den Einstellungen oder folge der Windows-Anzeigesprache. Änderungen gelten sofort und bleiben nach einem Neustart erhalten.

## Kompatibilität und Hardwarezugriff

- Für die Hardwaresteuerung sind Administratorrechte und ein kompatibler Lecoo-WMI-Anbieter erforderlich.
- Optional verfügbar sind Autostart bei der Anmeldung, Steuerung über den Infobereich, eine Instanz pro Sitzung sowie die Lüfterziele Auto/Maximum. Manuelle Ziele mit geringerer Lüfterleistung bleiben deaktiviert, bis die Temperaturzone zum CPU-Schutz validiert wurde.
- Die Firmware meldet gemessene RPM, liefert jedoch keine Rückmeldung zu Tastgrad oder Steuermodus. Siehe den [Funktionsleitfaden](docs/features.md).
- WinRing0 wird nicht verwendet. Ein zusätzlicher Sensortreiber, .NET oder eine separate VC++-Laufzeit ist nicht erforderlich.

## Download

Lade das Windows-x64-Paket aus dem [Release v0.1.0](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.1.0) herunter:

- `lecoo-rust-powercontrol-v0.1.0-windows-x64.zip` — Anwendung, Hardwaretestprogramm und Prüfsummen der ausführbaren Dateien.
- `lecoo-rust-powercontrol-v0.1.0-windows-x64-setup.exe` — Installationsprogramm mit Deinstallationsprogramm.

Führe `lecoo-control-center.exe` als Administrator aus, um auf die Hardwaresteuerung zuzugreifen. Für den automatischen Start aktiviere **Start at sign-in (minimized to tray)** in den Einstellungen; standardmäßig ist die Option ausgeschaltet. Validierung und bekannte Einschränkungen stehen in den [Release Notes v0.1.0](docs/releases/v0.1.0.md).

## Aus dem Quellcode bauen

Die Anwendung ist für 64-Bit-Windows ausgelegt. Installiere die stabile Rust-MSVCToolchain und Visual Studio Build Tools mit Windows SDK und führe anschließend Folgendes aus:

```powershell
cargo fetch --locked
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-release.ps1
```

Für die Paketierung wird außerdem Inno Setup 6 benötigt. Das Skript erstellt ZIP und Installationsprogramm in `dist/`. GitHub Actions erzeugt beide Formate für jedes Versions-Tag und prüft Installation und Entfernung. Die native C-Laufzeit ist statisch eingebunden.

## Automatisierter Hardwaretest

Auf einem kompatiblen Windows-System ausführen:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\test-power-modes.ps1
```

Das Skript fordert Administratorrechte an, vergleicht alle drei Profile mit einer unabhängigen Windows-CIM-Abfrage, prüft die Ablehnung ungültiger Moduswerte und stellt das ursprüngliche Profil nach Erfolg oder Fehler wieder her. Mit `-ReadOnly` lässt sich die Hardware ohne Änderung des Energieprofils prüfen. Detaillierte JSON-Berichte werden in `logs/power-mode-tests/` gespeichert; dieser Ordner ist vom öffentlichen Repository ausgeschlossen.

Bei jedem Branch-Push und Pull Request führt Windows CI Formatprüfung, Clippy, Unit-Tests für Bibliothek und Binärdatei sowie Release-Builds aus. Der Lauf kann auch manuell in Actions gestartet werden. Siehe den [Testleitfaden](docs/testing.md) und die [WMI-Schnittstellenreferenz](docs/reference/wmi/interfaces.md).

## Dokumentation

- [UI-Leitfaden](docs/native-ui.md) · [Funktionsleitfaden](docs/features.md) · [Testleitfaden](docs/testing.md)
- [WMI-Schnittstellenreferenz](docs/reference/wmi/interfaces.md) · [Release Notes v0.1.0](docs/releases/v0.1.0.md)

## Projektstruktur

| Pfad | Inhalt |
| --- | --- |
| `src/` | Rust-Anwendung, UI, Hardwareabstraktion und Testprogramm |
| `resources/` | Separate TOML-Oberflächenressourcen für alle acht Sprachen |
| `scripts/` | Windows-Build- und automatisierte Hardwaretestskripte |
| `docs/` | Einrichtung, Tests und Hardware-Schnittstellendokumentation |
| `.github/` | Windows-CI- und Release-Workflows |

## Lizenz

Es wird keine Open-Source-Lizenz gewährt. Das Urheberrecht liegt bei BlackSquarre; die öffentliche Verfügbarkeit des Repositorys erlaubt weder die Weiterverbreitung noch die Wiederverwendung des Codes.
