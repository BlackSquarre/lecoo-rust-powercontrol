# Windows UI and memory usage

The current source replaces the eframe/OpenGL UI with Win32 windows and controls. This interface is included in v0.0.3.

The layout follows the approved compact C revision, at 400 × 520 logical pixels by default. A two-by-two grid shows ACPI thermal-zone temperature, fan RPM, memory usage and disk usage. Power and fan controls sit underneath; quiet is green, balanced blue, performance red. The device footer contains the sole Settings gear. The endpoint/caption row and request/firmware/reconnect/version/connection information have been removed from the main dashboard. Settings opens an owned native window with sign-in startup, close behavior, language and About. Errors appear only when needed.

Temperature is read through Windows ACPI thermal-zone WMI. Package power and all WinRing0 access have been removed. An unavailable reading is displayed as Unavailable; the zone has not been validated as a CPU package sensor.

The language menu offers System default, 简体中文 and English. The default follows the Windows UI language (Chinese UI uses Simplified Chinese; other UI languages use English). Explicit language choices persist beside the close preference and update the dashboard, settings, About, close prompt and tray menu immediately. Theme remains independent of language and follows the Windows app light/dark setting. See [terminology and localization](localization.md).

The settings window uses a compact 380 × 312 logical-pixel layout, slightly narrower than the main window. Startup and Window & language have separate cards, with 16-pixel outer margins, 12-pixel card padding and a 12-pixel gap. Close behavior and language share aligned label and field columns in two adjacent rows. The startup status line, reset action, application-information label and startup parenthetical have been removed. Select Ask every time in the close-behavior menu to restore the close prompt. About sits directly below the cards, followed by an error area that only displays failures. The layout scales with Windows DPI and uses the existing system fonts and theme palette.

About shows the application icon, name, version, author, current local-calendar copyright year, Bilibili and project-website links, and third-party notices. The year updates while the window is open. Links open in the default browser; license text opens in a native read-only scrolling EDIT control. No embedded browser, custom graphics context or bitmap back buffer is allocated.

Auto and Maximum remain available. The Manual option and slider are disabled pending validation of a sensor for CPU protection. Worker heartbeat and RPM failure recovery remain active. A worker starts only when cooling controls are used.

Closing the main window initially opens a choice dialog: minimize to the notification area or exit, with a Remember checkbox. Choosing without Remember applies only once. Remembered choices are saved under `%LOCALAPPDATA%\LecooRustPowerControl\preferences.txt`. Select Ask every time in Settings to show the prompt again. Cancelling the dialog keeps the window open. The dialog uses the ongoing controller message loop so manual cooling heartbeats continue while it is open.

Minimizing or using the tray menu destroys the main window, child controls, fonts and owned icons. Opening from the tray or launching again recreates them. A small hidden controller window continues hardware monitoring and tray processing. Destroying the UI does not promise that every process heap page or shared system DLL is unmapped; no working-set trimming API is used.

The existing green power-button artwork is embedded as the executable's default icon, installer icon and shortcut icon. Running taskbar, title-bar, tray and owned-window icons follow actual power-mode readback: Quiet is green, Balanced blue, Performance red, and an unavailable reading gray. About paints a separate DPI-sized icon of the same color. Ten icon sizes are embedded as compressed PNG frames; no image decoding library or runtime artwork generator is added. Window icon handles are replaced only when mode or DPI changes and released on window destruction. The tray retains four shared native handles, updates on mode/language changes and checks registration every five seconds using the existing controller timer.

`scripts/test-mode-icons.ps1` runs the production native window/icon code with simulated mode values, without hardware writes, preference changes or interference with a running application. On 2026-10-04, 11 library tests, four preference tests and the native icon regression passed. All five window types were checked for all four colors and unchanged-refresh handle reuse. Twelve window creation/destruction cycles kept GDI/USER counts at 36/31, and 100/125/150/200% DPI icon dimensions and colors passed. These are short native-resource checks, not whole-app memory or CPU measurements. Local evidence is in `logs/mode-icons/native-icons.json`; assets and rebuilding instructions are in [assets/icons](../assets/icons/README.md).

## Historical A-layout validation

2026-10-02 on the verified Lecoo/8745H system, the optimized application SHA256 was `9A882D1DF6775E8D4FE74470BD882AB4309BDCEC396E8EEE68AE6938412E8A97`. After warmup, the visible dashboard had about 19.5 MiB total working set, 3.0 MiB private working set and 4.5 MiB private commit. Only the main process was running. For comparison, the earlier [published 0.0.2 measurements](memory-research.md) were 152.4 / 123.4 / 209.7 MiB respectively; these are separate runs, not a simultaneous comparison.

While hidden with a manual fan session, the main process and worker together had about 4.9 MiB private working set and 6.8 MiB private commit. Summed total working set was about 30.5 MiB, which double-counts some shared DLL pages. This state includes a worker, unlike the visible-dashboard sample. Eight minimize/reopen cycles held GDI and USER counts at 32 and 65, respectively. These short tests do not establish a long-term leak-free result.

`scripts/test-native-ui.ps1` exercises the release executable on the compatible machine. It changes power modes, temporary app preferences and the Windows app theme, and restores all three after testing. Do not run it against a user-owned running application.

It verifies independent power-mode readback, compact layout, thermal-zone readings, language persistence and About, light/dark client and title-bar colors, disabled manual controls, maximum fan requests, close-dialog heartbeat continuity, remembered choices, cancellation, tray window destruction/recreation and repeated GDI/USER handle counts. Current compact UI screenshots and JSON are stored in ignored `logs/compact-ui/`; prior A-layout evidence remains in `logs/native-migration/`. `scripts/test-features.ps1` additionally checks startup registration/action, single-instance wakeup, fan ranges, timeout and EOF recovery.

The original sketches are in [design/native-ui](design/native-ui/README.md); actual UI copy and placement follow subsequent user decisions.


## Historical compact C validation before v0.0.3 (2026-10-04)

Final local Release SHA256: `23BA69C5394CFBD3255287115FD2DDC6972CFCA86EFEABCB8081399D5088662D`; executable size 641,536 bytes. Eighteen unit tests passed. `logs/compact-ui/native-ui-final.json` and `features-final.json` both report Passed=true and Restored=true with this same binary hash.

Checks cover Chinese/English switching and process-restart persistence, immediate dashboard/settings/About translation, current-year copyright, the complete scrolling notices document, live CPU package watts, all three power-mode readbacks, system themes, fan-slider release/keyboard submission, manual fan heartbeat across Settings/About/notices and close prompts, remembered/reset/cancelled close choices, repeated UI recreation, startup-task setup/launch/cleanup, worker timeout and EOF recovery. Tests restore original preferences, theme, power profile and startup task; cooling ends with an automatic request and status 0. External website availability was not validated; the supplied links use the default browser.

Five warm visible samples averaged 19.50 MiB total working set, 2.96 MiB private working set and 4.35 MiB private commit. Eight minimize/reopen cycles kept GDI/USER counts at 38/48. This is a short local measurement, not proof of long-term leak freedom. Opening About/notices can allocate additional native resources; owned windows and their fonts/icons are released on close.
