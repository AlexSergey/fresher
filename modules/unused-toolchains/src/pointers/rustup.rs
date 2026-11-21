//! rustup: toolchain folder names, references (a settings value or a pin's channel), and the files that hold them.

use toml::{Table, Value};

use super::{numbers, without_bom};

/// A toolchain's channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Channel {
    Stable,
    Beta,
    Nightly,
    /// `X.Y` or `X.Y.Z`.
    Version(Vec<u32>),
}

/// A toolchain name: `<channel>[-<YYYY-MM-DD>][-<triple>]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ToolchainName {
    pub(crate) channel: Channel,
    pub(crate) date: Option<String>,
    pub(crate) triple: Option<String>,
}

/// One value of the `[overrides]` of a `settings.toml`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Override {
    /// A toolchain reference, to be read with [`reference`].
    Reference(String),
    /// A value that is not text: it cannot be read, so it is `Unknown` downstream.
    Unreadable,
}

/// What a `rust-toolchain.toml` or a legacy `rust-toolchain` names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FileRef {
    /// A channel reference, to be read with [`reference`].
    Channel(String),
    /// A linked toolchain path: no toolchain of rustup.
    Path,
    Unknown,
}

/// The name of a toolchain folder: its triple is required; any other name is off shape.
pub(crate) fn folder(name: &str) -> Option<ToolchainName> {
    parse(name).filter(|parsed| parsed.triple.is_some())
}

/// A reference: the same grammar with the triple optional.
pub(crate) fn reference(text: &str) -> Option<ToolchainName> {
    parse(text)
}

/// A named channel without a date: it moves, so it is never offered.
pub(crate) fn is_moving(name: &ToolchainName) -> bool {
    name.date.is_none() && !matches!(name.channel, Channel::Version(_))
}

/// Whether `reference` keeps `folder`: equal channels (or `X.Y` over `X.Y.Z`), equal dates, and a triple that is
/// absent or equal.
pub(crate) fn keeps(reference: &ToolchainName, folder: &ToolchainName) -> bool {
    let channel = match (&reference.channel, &folder.channel) {
        (Channel::Version(wanted), Channel::Version(found)) => {
            wanted == found || (wanted.len() == 2 && found.len() == 3 && found.starts_with(wanted))
        }
        (wanted, found) => wanted == found,
    };
    channel
        && reference.date == folder.date
        && reference.triple.as_ref().is_none_or(|triple| folder.triple.as_ref() == Some(triple))
}

/// The default toolchain and the overrides' values of a `settings.toml`. A value that is not text is
/// [`Override::Unreadable`]. The default is `None` when the text is not TOML or names none.
pub(crate) fn settings(text: &str) -> (Option<String>, Vec<Override>) {
    let Ok(table) = without_bom(text).parse::<Table>() else {
        return (None, Vec::new());
    };
    let default = table.get("default_toolchain").and_then(Value::as_str).map(str::to_owned);
    let overrides = match table.get("overrides") {
        None => Vec::new(),
        Some(Value::Table(overrides)) => overrides.values().map(override_of).collect(),
        Some(_) => vec![Override::Unreadable],
    };
    (default, overrides)
}

/// What a `rust-toolchain.toml` or the legacy `rust-toolchain` names.
pub(crate) fn toolchain_file(text: &str) -> FileRef {
    let text = without_bom(text);
    if let Some(Value::Table(toolchain)) = text.parse::<Table>().ok().as_ref().and_then(|table| table.get("toolchain"))
    {
        return match (toolchain.get("channel"), toolchain.get("path")) {
            (Some(Value::String(channel)), _) => FileRef::Channel(channel.clone()),
            (None, Some(Value::String(_))) => FileRef::Path,
            _ => FileRef::Unknown,
        };
    }
    let line = text.trim();
    if line.is_empty() || line.contains(char::is_whitespace) {
        FileRef::Unknown
    } else {
        FileRef::Channel(line.to_owned())
    }
}

fn override_of(value: &Value) -> Override {
    value.as_str().map_or(Override::Unreadable, |text| Override::Reference(text.to_owned()))
}

fn parse(text: &str) -> Option<ToolchainName> {
    let mut parts = text.split('-').peekable();
    let channel = match parts.next()? {
        "stable" => Channel::Stable,
        "beta" => Channel::Beta,
        "nightly" => Channel::Nightly,
        version => Channel::Version(version_numbers(version)?),
    };
    let mut date = None;
    if parts.peek().is_some_and(|part| part.bytes().all(|byte| byte.is_ascii_digit())) {
        let (year, month, day) = (parts.next()?, parts.next()?, parts.next()?);
        if year.len() != 4 || month.len() != 2 || day.len() != 2 || !day.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        if !month.bytes().all(|byte| byte.is_ascii_digit()) || matches!(channel, Channel::Version(_)) {
            return None;
        }
        date = Some(format!("{year}-{month}-{day}"));
    }
    let rest: Vec<&str> = parts.collect();
    let triple = match rest.len() {
        0 => None,
        3 | 4 if is_triple(&rest) => Some(rest.join("-")),
        _ => return None,
    };
    Some(ToolchainName { channel, date, triple })
}

/// `X.Y` or `X.Y.Z`, digits only.
fn version_numbers(text: &str) -> Option<Vec<u32>> {
    let numbers = numbers(text)?;
    (2..=3).contains(&numbers.len()).then_some(numbers)
}

/// Non-empty parts of `[a-z0-9_.]`; the architecture has no dot, which tells a triple from a pre-release suffix.
fn is_triple(parts: &[&str]) -> bool {
    parts.iter().all(|part| {
        !part.is_empty() && part.bytes().all(|byte| matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'.'))
    }) && !parts[0].contains('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder_of(name: &str) -> ToolchainName {
        folder(name).unwrap()
    }

    fn reference_of(text: &str) -> ToolchainName {
        reference(text).unwrap()
    }

    mod negative {
        use super::*;

        #[test]
        fn a_custom_or_beta_numbered_name_is_off_shape() {
            for name in [
                "my-toolchain",
                "1.83.0-beta.3-aarch64-apple-darwin",
                "stable",
                ".DS_Store",
                "Stable-aarch64-apple-darwin",
            ] {
                assert_eq!(folder(name), None, "{name}");
            }
        }

        #[test]
        fn a_bad_date_is_off_shape() {
            for name in [
                "nightly-2024-1-01-aarch64-apple-darwin",
                "nightly-24-01-01-aarch64-apple-darwin",
                "nightly-2024-01-1-aarch64-apple-darwin",
                "nightly-2024-aarch64-apple-darwin",
                "1.83.0-2024-01-01-aarch64-apple-darwin",
            ] {
                assert_eq!(folder(name), None, "{name}");
            }
        }

        #[test]
        fn a_folder_needs_its_triple() {
            assert_eq!(folder("nightly-2024-01-01"), None);
            assert_eq!(folder("1.83.0"), None);
            assert!(reference("nightly-2024-01-01").is_some());
        }

        #[test]
        fn a_reference_off_the_grammar_does_not_parse() {
            for text in ["", "1", "1.83.0-beta.3", "latest", "1.x", "stable-aarch64"] {
                assert_eq!(reference(text), None, "{text}");
            }
        }

        #[test]
        fn x_y_z_does_not_keep_x_y() {
            assert!(!keeps(&reference_of("1.83.0"), &folder_of("1.83-aarch64-apple-darwin")));
            assert!(!keeps(&reference_of("1.83.1"), &folder_of("1.83.0-aarch64-apple-darwin")));
            assert!(!keeps(&reference_of("1.84"), &folder_of("1.83.0-aarch64-apple-darwin")));
        }

        #[test]
        fn an_undated_reference_does_not_keep_a_dated_nightly() {
            assert!(!keeps(&reference_of("nightly"), &folder_of("nightly-2024-01-01-aarch64-apple-darwin")));
            assert!(!keeps(&reference_of("nightly-2024-01-02"), &folder_of("nightly-2024-01-01-aarch64-apple-darwin")));
        }

        #[test]
        fn another_triple_does_not_keep() {
            assert!(!keeps(&reference_of("stable-x86_64-apple-darwin"), &folder_of("stable-aarch64-apple-darwin")));
        }

        #[test]
        fn another_channel_does_not_keep() {
            assert!(!keeps(&reference_of("beta"), &folder_of("stable-aarch64-apple-darwin")));
        }

        #[test]
        fn settings_that_are_not_toml_have_no_default() {
            assert_eq!(settings("default_toolchain = = stable"), (None, vec![]));
            assert_eq!(settings("[overrides"), (None, vec![]));
        }

        #[test]
        fn settings_without_a_text_default_have_none() {
            assert_eq!(settings("version = \"12\"\n").0, None);
            assert_eq!(settings("default_toolchain = 3\n").0, None);
        }

        #[test]
        fn an_override_that_is_not_text_is_unreadable() {
            let (_, overrides) = settings("[overrides]\n\"/a\" = 3\n");

            assert_eq!(overrides, [Override::Unreadable]);
        }

        #[test]
        fn a_leading_bom_does_not_hide_a_file_s_content() {
            assert_eq!(toolchain_file("\u{feff}[toolchain]\nchannel = 3\n"), FileRef::Unknown);
            assert_eq!(toolchain_file("\u{feff}two words\n"), FileRef::Unknown);
        }

        #[test]
        fn a_toolchain_file_without_channel_or_path_is_unknown() {
            for text in [
                "",
                "[toolchain]\ncomponents = [\"rustfmt\"]\n",
                "[toolchain]\nchannel = 3\n",
                "two words\n",
                "[toolchain]\n",
            ] {
                assert_eq!(toolchain_file(text), FileRef::Unknown, "{text:?}");
            }
        }
    }

    mod positive {
        use proptest::prelude::*;

        use super::*;

        #[test]
        fn a_folder_is_its_channel_date_and_triple() {
            let name = folder_of("nightly-2024-01-01-aarch64-apple-darwin");

            assert_eq!(name.channel, Channel::Nightly);
            assert_eq!(name.date.as_deref(), Some("2024-01-01"));
            assert_eq!(name.triple.as_deref(), Some("aarch64-apple-darwin"));
            assert_eq!(folder_of("1.83.0-x86_64-unknown-linux-gnu").channel, Channel::Version(vec![1, 83, 0]));
        }

        #[test]
        fn stable_and_nightly_without_a_date_are_moving() {
            assert!(is_moving(&folder_of("stable-aarch64-apple-darwin")));
            assert!(is_moving(&folder_of("beta-aarch64-apple-darwin")));
            assert!(is_moving(&folder_of("nightly-aarch64-apple-darwin")));
        }

        #[test]
        fn a_dated_nightly_and_a_version_are_not_moving() {
            assert!(!is_moving(&folder_of("nightly-2024-01-01-aarch64-apple-darwin")));
            assert!(!is_moving(&folder_of("1.83.0-aarch64-apple-darwin")));
        }

        #[test]
        fn x_y_keeps_every_x_y_z() {
            assert!(keeps(&reference_of("1.83"), &folder_of("1.83.0-aarch64-apple-darwin")));
            assert!(keeps(&reference_of("1.83"), &folder_of("1.83.7-aarch64-apple-darwin")));
            assert!(keeps(&reference_of("1.83.0"), &folder_of("1.83.0-aarch64-apple-darwin")));
        }

        #[test]
        fn a_reference_without_a_triple_keeps_every_triple() {
            assert!(keeps(&reference_of("stable"), &folder_of("stable-aarch64-apple-darwin")));
            assert!(keeps(&reference_of("stable"), &folder_of("stable-x86_64-apple-darwin")));
            assert!(keeps(&reference_of("stable-aarch64-apple-darwin"), &folder_of("stable-aarch64-apple-darwin")));
            assert!(keeps(&reference_of("nightly-2024-01-01"), &folder_of("nightly-2024-01-01-aarch64-apple-darwin")));
        }

        #[test]
        fn settings_give_the_default_and_every_override() {
            let text = "default_toolchain = \"stable-aarch64-apple-darwin\"\nversion = \"12\"\n\n[overrides]\n\"/a\" = \"1.75.0\"\n\"/b\" = \"nightly-2024-01-01\"\n";

            let (default, overrides) = settings(text);

            assert_eq!(default.as_deref(), Some("stable-aarch64-apple-darwin"));
            assert_eq!(
                overrides,
                [Override::Reference("1.75.0".to_owned()), Override::Reference("nightly-2024-01-01".to_owned())]
            );
        }

        #[test]
        fn a_toolchain_file_gives_its_channel_or_path() {
            assert_eq!(
                toolchain_file("[toolchain]\nchannel = \"1.75.0\"\ncomponents = [\"clippy\"]\n"),
                FileRef::Channel("1.75.0".to_owned())
            );
            assert_eq!(toolchain_file("[toolchain]\npath = \"/opt/rust\"\n"), FileRef::Path);
        }

        #[test]
        fn a_bom_crlf_and_trailing_blanks_are_read_through() {
            assert_eq!(toolchain_file("\u{feff}1.75.0 \r\n"), FileRef::Channel("1.75.0".to_owned()));
            assert_eq!(
                toolchain_file("\u{feff}[toolchain]\r\nchannel = \"beta\"\r\n"),
                FileRef::Channel("beta".to_owned())
            );
            assert_eq!(settings("default_toolchain = \"stable\"\r\n").0.as_deref(), Some("stable"));
        }

        #[test]
        fn the_channel_wins_over_the_path() {
            assert_eq!(
                toolchain_file("[toolchain]\npath = \"/opt/rust\"\nchannel = \"1.75.0\"\n"),
                FileRef::Channel("1.75.0".to_owned())
            );
        }

        #[test]
        fn a_legacy_one_line_file_is_a_channel() {
            assert_eq!(toolchain_file("nightly-2024-01-01\n"), FileRef::Channel("nightly-2024-01-01".to_owned()));
            assert_eq!(toolchain_file("1.75.0"), FileRef::Channel("1.75.0".to_owned()));
        }

        proptest! {
            #[test]
            fn keeps_implies_equal_channel_and_date_or_the_x_y_prefix(
                reference_text in "(stable|beta|nightly|1\\.[0-9]{1,2}(\\.[0-9])?)(-2024-0[12]-01)?(-(a|b)-c-d)?",
                folder_text in "(stable|beta|nightly|1\\.[0-9]{1,2}\\.[0-9])(-2024-0[12]-01)?-(a|b)-c-d",
            ) {
                if let (Some(wanted), Some(found)) = (reference(&reference_text), folder(&folder_text))
                    && keeps(&wanted, &found)
                {
                        prop_assert_eq!(&wanted.date, &found.date);
                        let same = wanted.channel == found.channel;
                        let prefix = matches!(
                            (&wanted.channel, &found.channel),
                            (Channel::Version(x), Channel::Version(y)) if x.len() == 2 && y.starts_with(x)
                        );
                        prop_assert!(same || prefix);
                }
            }
        }
    }
}
