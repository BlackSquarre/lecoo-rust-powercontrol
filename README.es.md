# Lecoo Rust PowerControl

<p align="center">
  <strong>Un centro de control ligero para Windows, creado con Rust y compatible con sistemas Lecoo.</strong>
</p>

<p align="center">
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases"><img src="https://img.shields.io/github/v/release/BlackSquarre/lecoo-rust-powercontrol?display_name=tag" alt="Última versión"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/release.yml/badge.svg" alt="Compilación de versiones para Windows"></a>
  <a href="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/ci.yml"><img src="https://github.com/BlackSquarre/lecoo-rust-powercontrol/actions/workflows/ci.yml/badge.svg" alt="CI de Windows"></a>
  <img src="https://img.shields.io/badge/Rust-2021-orange?logo=rust" alt="Rust 2021">
  <img src="https://img.shields.io/badge/platform-Windows%20x64-blue" alt="Windows x64">
</p>

<p align="center">
  <a href="README.md">English</a> | <a href="README.zh-Hans.md">简体中文</a> | <a href="README.zh-Hant.md">繁體中文</a> | <a href="README.ja.md">日本語</a> | <a href="README.ko.md">한국어</a> | Español | <a href="README.fr.md">Français</a> | <a href="README.de.md">Deutsch</a>
</p>

**Autor:** [BlackSquarre](https://github.com/BlackSquarre) · **Versión actual:** [v0.1.0](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.1.0)

> Proyecto comunitario independiente, sin afiliación ni respaldo de Lecoo o Lenovo.

**Sistema verificado:** Lecoo MINI PRO-AHP · **CPU:** AMD Ryzen 7 8745H

## Funciones

- Cambia entre los perfiles de energía **Silencioso, Equilibrado y Rendimiento** y vuelve a leer el perfil activo del hardware después del cambio.
- Consulta la velocidad medida del ventilador, la temperatura de la zona térmica ACPI y el uso de memoria y disco.
- Interfaz nativa y compacta para Windows, con temas del sistema, ocho idiomas, ventana «Acerca de» y opción de cierre recordada. Consulta la [guía de la interfaz](docs/native-ui.md).
- La interfaz gráfica y la utilidad de pruebas de hardware usan la misma biblioteca de control escrita en Rust.

**Idiomas de la interfaz:** 简体中文, English, 繁體中文, 日本語, 한국어, Español, Français y Deutsch. Elige un idioma en Ajustes o sigue el idioma de visualización de Windows. Los cambios se aplican de inmediato y se conservan al reiniciar.

## Compatibilidad y acceso al hardware

- Los controles de hardware requieren privilegios de administrador y un proveedor WMI compatible de Lecoo.
- Se ofrecen inicio de sesión automático opcional, controles desde la bandeja, una instancia por sesión y objetivos de ventilador Auto/Maximum. Los objetivos manuales que reducen la velocidad están desactivados hasta validar la zona térmica para proteger la CPU.
- El firmware informa las RPM medidas, pero no permite leer el ciclo de trabajo ni el modo de control. Consulta la [guía de funciones](docs/features.md).
- No usa WinRing0 ni requiere instalar un controlador de sensores, .NET o un runtime VC++ independiente.

## Descarga

Descarga el paquete para Windows x64 desde la [versión v0.1.0](https://github.com/BlackSquarre/lecoo-rust-powercontrol/releases/tag/v0.1.0):

- `lecoo-rust-powercontrol-v0.1.0-windows-x64.zip`: aplicación, utilidad de pruebas de hardware y sumas de comprobación de los ejecutables.
- `lecoo-rust-powercontrol-v0.1.0-windows-x64-setup.exe`: instalador con desinstalador.

Ejecuta `lecoo-control-center.exe` como administrador para acceder a los controles de hardware. Para iniciar la aplicación automáticamente, activa **Start at sign-in (minimized to tray)** en Ajustes; está desactivado de forma predeterminada. Consulta las [notas de la versión v0.1.0](docs/releases/v0.1.0.md) para conocer la validación y las limitaciones.

## Compilar desde el código fuente

La aplicación está dirigida a Windows de 64 bits. Instala la cadena de herramientas estable Rust MSVC y Visual Studio Build Tools con Windows SDK; después ejecuta:

```powershell
cargo fetch --locked
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\build-release.ps1
```

El empaquetado también requiere Inno Setup 6. El script crea un ZIP y un instalador en `dist/`. GitHub Actions genera ambos formatos para cada etiqueta de versión y comprueba la instalación y la desinstalación. El runtime C nativo está enlazado estáticamente.

## Prueba automatizada de hardware

En un sistema Windows compatible, ejecuta:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\test-power-modes.ps1
```

El script solicita privilegios de administrador, compara los tres perfiles con una lectura independiente de Windows CIM, comprueba que se rechacen los valores de modo no válidos y restaura el perfil original tanto si termina correctamente como si falla. Usa `-ReadOnly` para consultar el hardware sin cambiar el perfil de energía. Los informes JSON detallados se guardan en `logs/power-mode-tests/`, una carpeta excluida del repositorio público.

Cada envío a una rama y cada Pull Request ejecutan Windows CI para revisar el formato, Clippy, las pruebas unitarias de la biblioteca y el binario, y las compilaciones de lanzamiento. También puede iniciarse manualmente desde Actions. Consulta la [guía de pruebas](docs/testing.md) y la [referencia de interfaces WMI](docs/reference/wmi/interfaces.md).

## Documentación

- [Guía de la interfaz](docs/native-ui.md) · [Guía de funciones](docs/features.md) · [Guía de pruebas](docs/testing.md)
- [Referencia de interfaces WMI](docs/reference/wmi/interfaces.md) · [Notas de la versión v0.1.0](docs/releases/v0.1.0.md)

## Estructura del proyecto

| Ruta | Contenido |
| --- | --- |
| `src/` | Aplicación Rust, interfaz, abstracción de hardware y herramienta de pruebas |
| `resources/` | Recursos de interfaz TOML independientes para los ocho idiomas |
| `scripts/` | Scripts de compilación para Windows y pruebas automatizadas de hardware |
| `docs/` | Configuración, pruebas y documentación de interfaces de hardware |
| `.github/` | Flujos de CI y lanzamientos para Windows |

## Licencia

No se concede ninguna licencia de código abierto. Los derechos de autor pertenecen a BlackSquarre; que el repositorio sea público no autoriza la redistribución ni la reutilización del código.
