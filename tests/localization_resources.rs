#[path = "../build_support/localization.rs"]
mod resource_compiler;

use resource_compiler::{parse, render, Catalog, LOCALES};

fn catalogs() -> Vec<Catalog> {
    [
        include_str!("../resources/locales/zh-Hans.toml"),
        include_str!("../resources/locales/en.toml"),
        include_str!("../resources/locales/zh-Hant.toml"),
        include_str!("../resources/locales/ja.toml"),
        include_str!("../resources/locales/ko.toml"),
        include_str!("../resources/locales/es.toml"),
        include_str!("../resources/locales/fr.toml"),
        include_str!("../resources/locales/de.toml"),
    ]
    .into_iter()
    .zip(LOCALES)
    .map(|(source, code)| parse(source, code).unwrap())
    .collect()
}

#[test]
fn standalone_catalogs_compile_with_preserved_names_and_utf8_text() {
    let catalogs = catalogs();
    assert_eq!(catalogs[0].native_name, "简体中文");
    assert_eq!(catalogs[2].native_name, "繁體中文");
    assert_eq!(catalogs[4].native_name, "한국어");
    assert_eq!(catalogs[5].native_name, "Español");
    assert_eq!(catalogs[6].messages["settings.title"], "Paramètres");
    assert!(render(&catalogs).unwrap().contains("SettingsTitle"));
}

#[test]
fn build_rejects_missing_and_unknown_translation_keys() {
    let mut catalogs = catalogs();
    let text = catalogs[2].messages.remove("settings.title").unwrap();
    let error = render(&catalogs).unwrap_err();
    assert!(error.contains("zh-Hant") && error.contains("settings.title"));
    catalogs[2].messages.insert("settings.title".into(), text);
    catalogs[6]
        .messages
        .insert("settings.typo".into(), "Erreur".into());
    let error = render(&catalogs).unwrap_err();
    assert!(error.contains("fr") && error.contains("settings.typo"));
}

#[test]
fn build_rejects_colliding_rust_message_identifiers() {
    let mut catalogs = catalogs();
    for catalog in &mut catalogs {
        catalog.messages.insert("test.a_b".into(), "One".into());
        catalog.messages.insert("test.a.b".into(), "Two".into());
    }
    assert!(render(&catalogs)
        .unwrap_err()
        .contains("duplicate Rust identifiers"));
}

#[test]
fn build_preserves_quotes_backslashes_and_line_breaks_in_translations() {
    let mut catalogs = catalogs();
    for catalog in &mut catalogs {
        catalog
            .messages
            .insert("test.escaped".into(), "\"quoted\"\nC:\\folder".into());
    }
    let generated = render(&catalogs).unwrap();
    assert!(generated.contains("\\\"quoted\\\"\\nC:\\\\folder"));
}

#[test]
fn parser_rejects_mismatched_codes_invalid_keys_and_invalid_text() {
    let english = include_str!("../resources/locales/en.toml");
    assert!(parse(english, "fr").unwrap_err().contains("does not match"));
    for replacement in [
        "\"settings.title\" = \"  \"",
        "\"settings.title\" = 42",
        "\"settings.title\" = \"\\u0000\"",
        "\"settings.title\" = \"\u{fffd}\"",
        "\"settings. invalid\" = \"Settings\"",
        "\"settings.title\" = \"Settings\"\n\"settings.title\" = \"Duplicate\"",
    ] {
        let changed = english.replace("\"settings.title\" = \"Settings\"", replacement);
        assert!(
            parse(&changed, "en").is_err(),
            "Accepted invalid resource: {replacement}"
        );
    }
    let changed = english.replace("native_name = \"English\"", "native_name = \"\"");
    assert!(parse(&changed, "en").is_err());
}

#[test]
fn build_rejects_language_order_mismatches() {
    let mut catalogs = catalogs();
    catalogs.swap(0, 1);
    assert!(render(&catalogs).is_err());
}
