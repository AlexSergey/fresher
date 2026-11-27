//! Reads `flags.toml` and the build's override into the flags a build ships with, and writes them as Rust (plan 043).
//! Compiled into the build script and, for its tests, into the app's lib; it must not depend on either.

use std::collections::HashSet;
use std::fmt;

/// Which column of `flags.toml` a build takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Profile {
    Release,
    Development,
}

/// One flag with the value this build ships.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Resolved {
    pub(crate) name: String,
    pub(crate) type_name: String,
    pub(crate) value: String,
}

/// Why a build's flags are refused; each names the flag and the key or the override's item.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FlagsError {
    NotToml(String),
    NotATable { flag: String },
    BadName { flag: String },
    BadType { flag: String, type_name: String },
    MissingKey { flag: String, key: &'static str },
    UnknownKey { flag: String, key: String },
    WrongKind { flag: String, key: &'static str },
    EmptyAbout { flag: String },
    TooFewValues { flag: String },
    RepeatedValue { flag: String, value: String },
    NotKebabCase { flag: String, value: String },
    NotAmongValues { flag: String, key: &'static str, value: String },
    OverrideWithoutEquals { item: String },
    OverrideUnknownFlag { flag: String },
    OverrideValueNotAllowed { flag: String, value: String },
    OverrideRepeated { flag: String },
}

impl fmt::Display for FlagsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotToml(why) => write!(f, "flags.toml is not valid TOML: {why}"),
            Self::NotATable { flag } => write!(f, "flags.toml: `{flag}` is not a table"),
            Self::BadName { flag } => write!(f, "flags.toml: `[{flag}]` is not a name of the form [a-z][a-z0-9]*"),
            Self::BadType { flag, type_name } => {
                write!(f, "flags.toml: `[{flag}]` `type` is `{type_name}`, which is not a PascalCase identifier")
            }
            Self::MissingKey { flag, key } => write!(f, "flags.toml: `[{flag}]` has no `{key}`"),
            Self::UnknownKey { flag, key } => write!(f, "flags.toml: `[{flag}]` has an unknown key `{key}`"),
            Self::WrongKind { flag, key } => write!(f, "flags.toml: `[{flag}]` `{key}` has the wrong kind of value"),
            Self::EmptyAbout { flag } => write!(f, "flags.toml: `[{flag}]` `about` is empty"),
            Self::TooFewValues { flag } => write!(f, "flags.toml: `[{flag}]` `values` needs at least two values"),
            Self::RepeatedValue { flag, value } => {
                write!(f, "flags.toml: `[{flag}]` `values` names `{value}` twice")
            }
            Self::NotKebabCase { flag, value } => {
                write!(f, "flags.toml: `[{flag}]` value `{value}` is not kebab-case")
            }
            Self::NotAmongValues { flag, key, value } => {
                write!(f, "flags.toml: `[{flag}]` `{key}` is `{value}`, which is not among its `values`")
            }
            Self::OverrideWithoutEquals { item } => {
                write!(f, "FRESHER_FLAGS: `{item}` has no `=`; write name=value")
            }
            Self::OverrideUnknownFlag { flag } => write!(f, "FRESHER_FLAGS: there is no flag `{flag}`"),
            Self::OverrideValueNotAllowed { flag, value } => {
                write!(f, "FRESHER_FLAGS: `{value}` is not a value of `{flag}`")
            }
            Self::OverrideRepeated { flag } => write!(f, "FRESHER_FLAGS: `{flag}` is named twice"),
        }
    }
}

impl std::error::Error for FlagsError {}

const KEYS: [&str; 5] = ["about", "type", "values", "release", "development"];

/// The flags of `file` for `profile`, `overrides` (`FRESHER_FLAGS`: comma-separated `name=value`) applied. The flags
/// come in the order of their names (the reader keeps its tables sorted).
pub(crate) fn resolve(file: &str, profile: Profile, overrides: Option<&str>) -> Result<Vec<Resolved>, FlagsError> {
    let root: toml::Table = file.parse().map_err(|error: toml::de::Error| FlagsError::NotToml(error.to_string()))?;
    let mut flags = Vec::new();
    let mut allowed: Vec<Vec<String>> = Vec::new();
    for (name, table) in &root {
        let toml::Value::Table(table) = table else {
            return Err(FlagsError::NotATable { flag: name.clone() });
        };
        let (resolved, values) = read_flag(name, table, profile)?;
        flags.push(resolved);
        allowed.push(values);
    }
    apply_overrides(&mut flags, &allowed, overrides.unwrap_or(""))?;
    Ok(flags)
}

fn read_flag(name: &str, table: &toml::Table, profile: Profile) -> Result<(Resolved, Vec<String>), FlagsError> {
    if !is_flag_name(name) {
        return Err(FlagsError::BadName { flag: name.to_owned() });
    }
    for key in table.keys() {
        if !KEYS.contains(&key.as_str()) {
            return Err(FlagsError::UnknownKey { flag: name.to_owned(), key: key.clone() });
        }
    }
    let text = |key: &'static str| -> Result<&str, FlagsError> {
        match table.get(key) {
            None => Err(FlagsError::MissingKey { flag: name.to_owned(), key }),
            Some(toml::Value::String(text)) => Ok(text),
            Some(_) => Err(FlagsError::WrongKind { flag: name.to_owned(), key }),
        }
    };
    let about = text("about")?;
    let type_name = text("type")?;
    let release = text("release")?;
    let development = text("development")?;
    let values = match table.get("values") {
        None => return Err(FlagsError::MissingKey { flag: name.to_owned(), key: "values" }),
        Some(toml::Value::Array(items)) => items
            .iter()
            .map(|item| item.as_str().map(str::to_owned))
            .collect::<Option<Vec<String>>>()
            .ok_or_else(|| FlagsError::WrongKind { flag: name.to_owned(), key: "values" })?,
        Some(_) => return Err(FlagsError::WrongKind { flag: name.to_owned(), key: "values" }),
    };
    if !is_type_name(type_name) {
        return Err(FlagsError::BadType { flag: name.to_owned(), type_name: type_name.to_owned() });
    }
    if about.trim().is_empty() {
        return Err(FlagsError::EmptyAbout { flag: name.to_owned() });
    }
    if values.len() < 2 {
        return Err(FlagsError::TooFewValues { flag: name.to_owned() });
    }
    let mut seen = HashSet::new();
    for value in &values {
        if !is_kebab_case(value) {
            return Err(FlagsError::NotKebabCase { flag: name.to_owned(), value: value.clone() });
        }
        if !seen.insert(value.as_str()) {
            return Err(FlagsError::RepeatedValue { flag: name.to_owned(), value: value.clone() });
        }
    }
    for (key, value) in [("release", release), ("development", development)] {
        if !values.iter().any(|allowed| allowed == value) {
            return Err(FlagsError::NotAmongValues { flag: name.to_owned(), key, value: value.to_owned() });
        }
    }
    let value = match profile {
        Profile::Release => release,
        Profile::Development => development,
    };
    Ok((Resolved { name: name.to_owned(), type_name: type_name.to_owned(), value: value.to_owned() }, values))
}

fn apply_overrides(flags: &mut [Resolved], allowed: &[Vec<String>], overrides: &str) -> Result<(), FlagsError> {
    let mut named = HashSet::new();
    for item in overrides.split(',').map(str::trim).filter(|item| !item.is_empty()) {
        let Some((flag, value)) = item.split_once('=') else {
            return Err(FlagsError::OverrideWithoutEquals { item: item.to_owned() });
        };
        let (flag, value) = (flag.trim(), value.trim());
        let Some(index) = flags.iter().position(|resolved| resolved.name == flag) else {
            return Err(FlagsError::OverrideUnknownFlag { flag: flag.to_owned() });
        };
        if !named.insert(flag.to_owned()) {
            return Err(FlagsError::OverrideRepeated { flag: flag.to_owned() });
        }
        if !allowed[index].iter().any(|candidate| candidate == value) {
            return Err(FlagsError::OverrideValueNotAllowed { flag: flag.to_owned(), value: value.to_owned() });
        }
        value.clone_into(&mut flags[index].value);
    }
    Ok(())
}

fn is_flag_name(value: &str) -> bool {
    value.starts_with(|c: char| c.is_ascii_lowercase())
        && value.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

fn is_type_name(value: &str) -> bool {
    value.starts_with(|c: char| c.is_ascii_uppercase()) && value.chars().all(|c| c.is_ascii_alphanumeric())
}

fn is_kebab_case(value: &str) -> bool {
    !value.is_empty()
        && value.split('-').all(|word| {
            !word.is_empty()
                && word.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
                && word.starts_with(|c: char| c.is_ascii_lowercase())
        })
}

fn pascal_case(value: &str) -> String {
    value
        .split('-')
        .map(|word| {
            let mut letters = word.chars();
            letters.next().map(|first| first.to_ascii_uppercase().to_string() + letters.as_str()).unwrap_or_default()
        })
        .collect()
}

/// The generated Rust: `FLAGS`, each field the variant of the flag's value.
pub(crate) fn render(flags: &[Resolved]) -> String {
    let fields: Vec<String> = flags
        .iter()
        .map(|flag| format!("    {}: fresher_core::{}::{},\n", flag.name, flag.type_name, pascal_case(&flag.value)))
        .collect();
    let fields = fields.concat();
    format!(
        "// Generated by app/tauri/build.rs from flags.toml and FRESHER_FLAGS. Do not edit.\n\
         pub(crate) const FLAGS: fresher_core::BuildFlags = fresher_core::BuildFlags {{\n{fields}}};\n"
    )
}

#[cfg(test)]
mod tests {
    const FILE: &str = r#"
[high]
about = "Whether High is offered."
type = "HighLevel"
values = ["offered", "withheld"]
release = "withheld"
development = "offered"

[largest]
about = "Whether Largest is shown."
type = "LargestScreen"
values = ["shown", "hidden"]
release = "hidden"
development = "shown"
"#;

    mod negative {
        use super::super::{FlagsError, Profile, resolve};
        use super::FILE;

        fn refused(file: &str) -> FlagsError {
            resolve(file, Profile::Release, None).unwrap_err()
        }

        fn flag(body: &str) -> String {
            format!("[a]\n{body}")
        }

        const GOOD: &str =
            "about = \"x\"\ntype = \"T\"\nvalues = [\"on\", \"off\"]\nrelease = \"on\"\ndevelopment = \"off\"\n";

        #[test]
        fn a_file_that_is_not_toml_is_refused() {
            assert!(matches!(refused("[high\nabout ="), FlagsError::NotToml(_)));
        }

        #[test]
        fn a_table_missing_a_key_is_refused() {
            for key in super::super::KEYS {
                let lines: Vec<&str> = GOOD.lines().filter(|line| !line.starts_with(key)).collect();
                let body = lines.join("\n");

                assert_eq!(refused(&flag(&body)), FlagsError::MissingKey { flag: "a".into(), key });
            }
        }

        #[test]
        fn a_value_of_the_wrong_kind_is_refused() {
            let about = GOOD.replace("about = \"x\"", "about = 1");
            let values = GOOD.replace("[\"on\", \"off\"]", "[1, 2]");

            assert_eq!(refused(&flag(&about)), FlagsError::WrongKind { flag: "a".into(), key: "about" });
            assert_eq!(refused(&flag(&values)), FlagsError::WrongKind { flag: "a".into(), key: "values" });
        }

        #[test]
        fn something_that_is_not_a_table_is_refused() {
            assert_eq!(refused("x = 1\n"), FlagsError::NotATable { flag: "x".into() });
        }

        #[test]
        fn a_table_with_another_key_is_refused() {
            let error = refused(&flag(&format!("{GOOD}colour = \"red\"\n")));

            assert_eq!(error, FlagsError::UnknownKey { flag: "a".into(), key: "colour".into() });
        }

        #[test]
        fn a_flag_name_or_a_type_that_is_not_an_identifier_is_refused() {
            for bad in ["Bad", "a-b", "a_b", "1a"] {
                let error = refused(&format!("[{bad:?}]\n{GOOD}"));

                assert_eq!(error, FlagsError::BadName { flag: bad.into() }, "{bad:?}");
            }
            for bad in ["t", "T-x", "T_x", "1T"] {
                let error = refused(&flag(&GOOD.replace("type = \"T\"", &format!("type = \"{bad}\""))));

                assert_eq!(error, FlagsError::BadType { flag: "a".into(), type_name: bad.into() }, "{bad:?}");
                assert!(error.to_string().contains(bad));
            }
        }

        #[test]
        fn an_empty_about_is_refused() {
            let error = refused(&flag(&GOOD.replace("about = \"x\"", "about = \" \"")));

            assert_eq!(error, FlagsError::EmptyAbout { flag: "a".into() });
        }

        #[test]
        fn fewer_than_two_values_or_a_repeated_one_is_refused() {
            let one =
                GOOD.replace("[\"on\", \"off\"]", "[\"on\"]").replace("development = \"off\"", "development = \"on\"");
            let twice = GOOD.replace("[\"on\", \"off\"]", "[\"on\", \"on\"]");

            assert_eq!(refused(&flag(&one)), FlagsError::TooFewValues { flag: "a".into() });
            assert_eq!(refused(&flag(&twice)), FlagsError::RepeatedValue { flag: "a".into(), value: "on".into() });
        }

        #[test]
        fn a_value_that_is_not_kebab_case_is_refused() {
            for bad in ["On", "on_off", "-on", "on-", "on--off", "1on", ""] {
                let file =
                    GOOD.replace("[\"on\", \"off\"]", &format!("[\"{bad}\", \"off\"]")).replace("\"on\"", "\"off\"");

                assert_eq!(
                    refused(&flag(&file)),
                    FlagsError::NotKebabCase { flag: "a".into(), value: bad.into() },
                    "{bad:?}"
                );
            }
        }

        #[test]
        fn a_release_or_development_value_outside_its_values_is_refused() {
            let release = GOOD.replace("release = \"on\"", "release = \"maybe\"");
            let development = GOOD.replace("development = \"off\"", "development = \"maybe\"");

            assert_eq!(
                refused(&flag(&release)),
                FlagsError::NotAmongValues { flag: "a".into(), key: "release", value: "maybe".into() }
            );
            assert_eq!(
                refused(&flag(&development)),
                FlagsError::NotAmongValues { flag: "a".into(), key: "development", value: "maybe".into() }
            );
        }

        #[test]
        fn an_override_naming_an_unknown_flag_is_refused() {
            let error = resolve(FILE, Profile::Release, Some("nope=shown")).unwrap_err();

            assert_eq!(error, FlagsError::OverrideUnknownFlag { flag: "nope".into() });
        }

        #[test]
        fn an_override_with_a_value_outside_its_values_is_refused() {
            let error = resolve(FILE, Profile::Release, Some("largest=nope")).unwrap_err();

            assert_eq!(error, FlagsError::OverrideValueNotAllowed { flag: "largest".into(), value: "nope".into() });
            assert!(error.to_string().contains("largest") && error.to_string().contains("nope"));
        }

        #[test]
        fn an_override_naming_a_flag_twice_or_without_an_equals_sign_is_refused() {
            let twice = resolve(FILE, Profile::Release, Some("largest=shown,largest=hidden")).unwrap_err();
            let bare = resolve(FILE, Profile::Release, Some("largest")).unwrap_err();

            assert_eq!(twice, FlagsError::OverrideRepeated { flag: "largest".into() });
            assert_eq!(bare, FlagsError::OverrideWithoutEquals { item: "largest".into() });
        }
    }

    mod positive {
        use super::super::{Profile, Resolved, render, resolve};
        use super::FILE;

        fn values(flags: &[Resolved]) -> Vec<(&str, &str)> {
            flags.iter().map(|f| (f.name.as_str(), f.value.as_str())).collect()
        }

        #[test]
        fn a_release_build_takes_the_release_column() {
            let flags = resolve(FILE, Profile::Release, None).unwrap();

            assert_eq!(values(&flags), [("high", "withheld"), ("largest", "hidden")]);
            assert_eq!(flags[0].type_name, "HighLevel");
        }

        #[test]
        fn a_development_build_takes_the_development_column() {
            let flags = resolve(FILE, Profile::Development, None).unwrap();

            assert_eq!(values(&flags), [("high", "offered"), ("largest", "shown")]);
        }

        #[test]
        fn an_override_replaces_only_the_flags_it_names() {
            let flags = resolve(FILE, Profile::Release, Some(" largest = shown ")).unwrap();

            assert_eq!(values(&flags), [("high", "withheld"), ("largest", "shown")]);
        }

        #[test]
        fn an_empty_override_changes_nothing() {
            let plain = resolve(FILE, Profile::Release, None).unwrap();

            assert_eq!(resolve(FILE, Profile::Release, Some("")).unwrap(), plain);
            assert_eq!(resolve(FILE, Profile::Release, Some(" , ")).unwrap(), plain);
        }

        #[test]
        fn render_writes_each_value_as_its_pascal_case_variant() {
            let flags = [
                Resolved { name: "high".into(), type_name: "HighLevel".into(), value: "withheld".into() },
                Resolved { name: "safe".into(), type_name: "SafeMode".into(), value: "only-safe".into() },
            ];

            assert_eq!(
                render(&flags),
                "// Generated by app/tauri/build.rs from flags.toml and FRESHER_FLAGS. Do not edit.\n\
                 pub(crate) const FLAGS: fresher_core::BuildFlags = fresher_core::BuildFlags {\n\
                 \x20   high: fresher_core::HighLevel::Withheld,\n\
                 \x20   safe: fresher_core::SafeMode::OnlySafe,\n\
                 };\n"
            );
        }

        #[test]
        fn the_repository_file_resolves_for_both_profiles() {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../flags.toml");
            let file = std::fs::read_to_string(path).unwrap();

            for profile in [Profile::Release, Profile::Development] {
                let flags = resolve(&file, profile, None).unwrap();

                assert_eq!(flags.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), ["high", "largest"]);
            }
        }
    }
}
