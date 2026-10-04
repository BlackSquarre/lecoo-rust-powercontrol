# Lecoo Rust PowerControl

<p align="center">
  <strong>Un centre de contrôle Windows léger, développé en Rust pour les systèmes Lecoo compatibles.</strong>
</p>

<p align="center">
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases"><img src="https://img.shields.io/github/v/release/BlackSquarre/lecoo-rust-powercontrol?display_name=tag" alt="Dernière version"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml/badge.svg" alt="Compilation des versions Windows"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/ci.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/ci.yml/badge.svg" alt="CI Windows"></a>
  <img src="https://img.shields.io/badge/Rust-2021-orange?logo=rust" alt="Rust 2021">
  <img src="https://img.shields.io/badge/platform-Windows%20x64-blue" alt="Windows x64">
</p>

<p align="center">
  <a href="README.md">English</a> | <a href="README.zh-Hans.md">简体中文</a> | <a href="README.zh-Hant.md">繁體中文</a> | <a href="README.ja.md">日本語</a> | <a href="README.ko.md">한국어</a> | <a href="README.es.md">Español</a> | Français | <a href="README.de.md">Deutsch</a>
</p>

**Auteur :** [BlackSquarre](https://github.com/BlackSquarre) · **Version actuelle :** [v0.1.0](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.1.0)

> Projet communautaire indépendant, sans affiliation ni soutien de Lecoo ou Lenovo.

**Système vérifié :** Lecoo MINI PRO-AHP · **CPU :** AMD Ryzen 7 8745H

## Fonctionnalités

- Basculez entre les profils d’alimentation **Silencieux, Équilibré et Performances**, puis relisez le profil actif depuis le matériel.
- Consultez la vitesse mesurée du ventilateur, la température de la zone ACPI et l’utilisation de la mémoire et du disque.
- Interface Windows native et compacte, avec thèmes système, huit langues, fenêtre « À propos » et mémorisation du choix de fermeture. Consultez le [guide de l’interface](docs/native-ui.md).
- L’interface graphique et l’outil de test matériel utilisent la même bibliothèque de contrôle écrite en Rust.

**Langues de l’interface :** 简体中文, English, 繁體中文, 日本語, 한국어, Español, Français et Deutsch. Choisissez une langue dans les paramètres ou suivez la langue d’affichage de Windows. Les changements sont immédiats et conservés après redémarrage.

## Compatibilité et accès au matériel

- Les commandes matérielles nécessitent des droits administrateur et un fournisseur WMI Lecoo compatible.
- Le démarrage à la connexion, les commandes depuis la zone de notification, une seule instance par session et les cibles de ventilateur Auto/Maximum sont disponibles en option. Les cibles manuelles qui réduisent la ventilation restent désactivées tant que la zone thermique n’est pas validée pour protéger le CPU.
- Le firmware indique les RPM mesurés, mais ne permet pas de relire le rapport cyclique ni le mode de contrôle. Consultez le [guide des fonctionnalités](docs/features.md).
- Aucun accès à WinRing0 ni installation de pilote de capteur, de .NET ou d’un runtime VC++ séparé n’est nécessaire.

## Téléchargement

Téléchargez le paquet Windows x64 depuis la [version v0.1.0](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.1.0) :

- `lecoo-rust-powercontrol-v0.1.0-windows-x64.zip` — application, outil de test matériel et sommes de contrôle des exécutables.
- `lecoo-rust-powercontrol-v0.1.0-windows-x64-setup.exe` — installateur avec programme de désinstallation.

Exécutez `lecoo-control-center.exe` en tant qu’administrateur pour accéder aux commandes matérielles. Pour le démarrage automatique, activez **Start at sign-in (minimized to tray)** dans les paramètres ; l’option est désactivée par défaut. Consultez les [notes de version v0.1.0](docs/releases/v0.1.0.md) pour les validations et limites connues.

## Compiler depuis les sources

L’application cible Windows 64 bits. Installez la chaîne stable Rust MSVC et Visual Studio Build Tools avec le Windows SDK, puis exécutez :

```powershell
cargo fetch --locked
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-release.ps1
```

La création des paquets nécessite aussi Inno Setup 6. Le script crée un ZIP et un installateur dans `dist/`. GitHub Actions produit les deux formats pour chaque étiquette de version et vérifie l’installation et la désinstallation. Le runtime C natif est lié statiquement.

## Test matériel automatisé

Sur un système Windows compatible, exécutez :

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\test-power-modes.ps1
```

Le script demande les droits administrateur, compare les trois profils à une lecture indépendante de Windows CIM, vérifie le rejet des valeurs de mode invalides et restaure le profil initial après réussite ou échec. Utilisez `-ReadOnly` pour inspecter le matériel sans changer le profil. Les rapports JSON détaillés sont écrits dans `logs/power-mode-tests/`, exclu du dépôt public.

Chaque push de branche et chaque Pull Request déclenchent la CI Windows : formatage, Clippy, tests unitaires de la bibliothèque et du binaire, et builds de publication. Elle peut aussi être lancée manuellement depuis Actions. Consultez le [guide de test](docs/testing.md) et la [référence des interfaces WMI](docs/reference/wmi/interfaces.md).

## Documentation

- [Guide de l’interface](docs/native-ui.md) · [Guide des fonctionnalités](docs/features.md) · [Guide de test](docs/testing.md)
- [Référence des interfaces WMI](docs/reference/wmi/interfaces.md) · [Notes de version v0.1.0](docs/releases/v0.1.0.md)

## Structure du projet

| Chemin | Contenu |
| --- | --- |
| `src/` | Application Rust, interface, abstraction matérielle et outil de test |
| `resources/` | Ressources d’interface TOML indépendantes dans les huit langues |
| `scripts/` | Scripts Windows de compilation et de test matériel automatisé |
| `docs/` | Préparation, tests et documentation des interfaces matérielles |
| `.github/` | Workflows CI et publication Windows |

## Licence

Aucune licence open source n’est accordée. Les droits d’auteur restent la propriété de BlackSquarre ; la visibilité publique du dépôt n’autorise pas la redistribution ni la réutilisation du code.
