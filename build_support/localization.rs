//! Validate standalone catalogs and generate static, typed resources at build time.
use std::collections::{BTreeMap, BTreeSet};

// Explicit language order matches Language::ALL, excluding System.
pub const LOCALES: [&str; 8] = ["zh-Hans", "en", "zh-Hant", "ja", "ko", "es", "fr", "de"];

#[derive(Debug, Clone)]
pub struct Catalog {
    pub code: String,
    pub native_name: String,
    pub messages: BTreeMap<String, String>,
}

pub fn parse(source: &str, expected_code: &str) -> Result<Catalog, String> {
    let document: toml::Value = source.parse().map_err(|error| format!("{error}"))?;
    let root = document.as_table().ok_or("Expected a TOML table")?;
    if root.len() != 2 || !root.contains_key("language") || !root.contains_key("messages") {
        return Err("Expected only [language] and [messages] tables".into());
    }
    let language = root["language"]
        .as_table()
        .ok_or("Invalid [language] table")?;
    if language.len() != 2 {
        return Err("Expected language.code and language.native_name".into());
    }
    let code = language
        .get("code")
        .and_then(toml::Value::as_str)
        .ok_or("Missing language.code")?;
    if code != expected_code {
        return Err(format!(
            "Language code {code:?} does not match {expected_code:?}"
        ));
    }
    let native_name = language
        .get("native_name")
        .and_then(toml::Value::as_str)
        .ok_or("Missing language.native_name")?;
    validate_text("language.native_name", native_name)?;
    let table = root["messages"]
        .as_table()
        .ok_or("Invalid [messages] table")?;
    if table.is_empty() {
        return Err("The message catalog is empty".into());
    }
    let mut messages = BTreeMap::new();
    for (key, value) in table {
        if !valid_key(key) {
            return Err(format!("Invalid message key: {key:?}"));
        }
        let text = value
            .as_str()
            .ok_or_else(|| format!("Message {key:?} must be a string"))?;
        validate_text(key, text)?;
        messages.insert(key.clone(), text.to_owned());
    }
    Ok(Catalog {
        code: code.to_owned(),
        native_name: native_name.to_owned(),
        messages,
    })
}

fn validate_text(key: &str, text: &str) -> Result<(), String> {
    if text.trim().is_empty() || text.contains(['\0', '\u{fffd}']) {
        return Err(format!("Empty or invalid text for {key:?}"));
    }
    Ok(())
}

fn valid_key(key: &str) -> bool {
    key.split('.').all(|segment| {
        segment.starts_with(|c: char| c.is_ascii_lowercase())
            && segment
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    })
}

fn variant_name(key: &str) -> String {
    key.split(['.', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut name = part.to_owned();
            name[..1].make_ascii_uppercase();
            name
        })
        .collect()
}

pub fn render(catalogs: &[Catalog]) -> Result<String, String> {
    if catalogs.len() != LOCALES.len()
        || catalogs
            .iter()
            .zip(LOCALES)
            .any(|(catalog, code)| catalog.code != code)
    {
        return Err("Catalog language order must match LOCALES".into());
    }
    let keys: BTreeSet<_> = catalogs[1].messages.keys().collect();
    let mut variants = BTreeSet::new();
    for key in &keys {
        if !variants.insert(variant_name(key)) {
            return Err(format!(
                "Message keys produce duplicate Rust identifiers: {key}"
            ));
        }
    }
    for catalog in catalogs {
        let actual: BTreeSet<_> = catalog.messages.keys().collect();
        if actual != keys {
            let missing: Vec<_> = keys.difference(&actual).collect();
            let extra: Vec<_> = actual.difference(&keys).collect();
            return Err(format!(
                "{}: missing keys {missing:?}; extra keys {extra:?}",
                catalog.code
            ));
        }
    }

    let count = keys.len();
    let mut output = String::from("// Generated from resources/locales/*.toml. Do not edit.\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n#[repr(usize)]\npub enum Text {\n");
    for key in &keys {
        output.push_str(&format!("    {},\n", variant_name(key)));
    }
    output.push_str(&format!(
        "}}\nimpl Text {{\n    pub const ALL: [Self; {count}] = [\n"
    ));
    for key in &keys {
        output.push_str(&format!("        Self::{},\n", variant_name(key)));
    }
    output.push_str(
        "    ];\n    pub fn key(self) -> &'static str { MESSAGE_KEYS[self as usize] }\n}\n",
    );
    output.push_str(&format!("static MESSAGE_KEYS: [&str; {count}] = [\n"));
    for key in &keys {
        output.push_str(&format!("    {key:?},\n"));
    }
    output.push_str("];\npub(super) static LANGUAGE_NAMES: [&str; 8] = [\n");
    for catalog in catalogs {
        output.push_str(&format!("    {:?},\n", catalog.native_name));
    }
    output.push_str(&format!(
        "];\npub(super) static MESSAGES: [[&str; {count}]; 8] = [\n"
    ));
    for catalog in catalogs {
        output.push_str("    [\n");
        for key in &keys {
            output.push_str(&format!("        {:?},\n", catalog.messages[*key]));
        }
        output.push_str("    ],\n");
    }
    output.push_str("];\n");
    Ok(output)
}

#[cfg(not(test))]
pub fn generate() {
    use std::{env, fs, path::PathBuf};
    let directory =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("resources/locales");
    println!("cargo:rerun-if-changed={}", directory.display());
    println!("cargo:rerun-if-changed=build_support/localization.rs");
    let catalogs: Vec<_> = LOCALES
        .into_iter()
        .map(|code| {
            let path = directory.join(format!("{code}.toml"));
            println!("cargo:rerun-if-changed={}", path.display());
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            parse(&source, code).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
        })
        .collect();
    let output =
        render(&catalogs).unwrap_or_else(|error| panic!("Invalid language resources: {error}"));
    let path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("localization_resources.rs");
    fs::write(path, output).expect("Unable to write generated localization resources");
}
