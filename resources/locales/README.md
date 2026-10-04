# Interface language resources

Each language has its own UTF-8 TOML file. All files must contain the same message keys:

| File | Language |
| --- | --- |
| `zh-Hans.toml` | 简体中文 |
| `en.toml` | English |
| `zh-Hant.toml` | 繁體中文 |
| `ja.toml` | 日本語 |
| `ko.toml` | 한국어 |
| `es.toml` | Español |
| `fr.toml` | Français |
| `de.toml` | Deutsch |

Example:

```toml
[language]
code = "en"
native_name = "English"

[messages]
"settings.title" = "Settings"
"settings.language_label" = "Language"
```

`language.code` must match the filename. `native_name` is displayed in the language menu. Message keys use lowercase letters, digits, underscores, and dots, such as `settings.title`. Values must be nonempty strings and may use TOML escape sequences.

`build_support/localization.rs` parses all catalogs at build time, checks missing keys, extra keys, and invalid values against `en.toml`, and generates the `Text` enum and static resource arrays. The UI uses `tr(Text::SettingsTitle)` to retrieve text in the current language. Invalid key references fail at compile time.

To change an existing translation, edit the corresponding catalog without changing the Rust UI code. Resources are embedded at build time, so rebuild the application after editing them. The application does not read these TOML files at runtime; portable and installed builds require no separate language files or runtime parser.

To add a message, add the same key to all eight catalogs. The build generates its enum variant automatically. To add a language, also update `Language`, the Windows display-language mapping, the build compiler's `LOCALES` order, and the catalog list in the tests.

Brand names, author credits, device models, technical units, and original third-party license text remain unchanged. Error classification converts raw hardware and worker diagnostics into the user-facing messages in these catalogs.
