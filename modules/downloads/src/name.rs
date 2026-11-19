//! The name rule (design, section 3.4): whether an installer's file name names exactly one installed app, and the
//! version it carries. Pure: on the file's stem, the installed apps and the installer words.

use std::cmp::Ordering;

use fresher_core::InstalledApp;

use crate::installers::Words;

/// A form shorter than this, in characters, names nothing: `go` would name too many files.
const SHORTEST_FORM: usize = 3;

/// Apple's own apps are never named: their installers are not downloaded.
const APPLE: &str = "com.apple.";

/// What an installer's stem names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Named {
    /// No installed app: the user's file.
    None,
    /// The app at `app` in the list, with the installer's version when its name carries one.
    One { app: usize, version: Option<Version> },
    /// Two apps or more: nothing is proven.
    Several,
}

/// A version of plain numbers (`133.0.1`), compared part by part, a missing part counting as 0 (`1` = `1.0`).
#[derive(Clone, Debug)]
pub(crate) struct Version(Vec<u32>);

impl Version {
    /// A version token of a file name: digits and dots with at least one dot, an optional leading `v`.
    pub(crate) fn parse(token: &str) -> Option<Self> {
        let digits = token.strip_prefix(['v', 'V']).unwrap_or(token);
        if !digits.contains('.') {
            return None;
        }
        Self::of_parts(digits)
    }

    /// An installed app's version: `None` when it is not only digits and dots (`1.0b2`).
    pub(crate) fn of_app(text: &str) -> Option<Self> {
        Self::of_parts(text)
    }

    /// Numbers between dots, none empty.
    fn of_parts(text: &str) -> Option<Self> {
        text.split('.')
            .map(|part| {
                if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
                    return None;
                }
                part.parse().ok()
            })
            .collect::<Option<Vec<u32>>>()
            .map(Self)
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let parts = self.0.len().max(other.0.len());
        (0..parts)
            .map(|at| {
                let part = |version: &Self| version.0.get(at).copied().unwrap_or(0);
                part(self).cmp(&part(other))
            })
            .find(|order| order.is_ne())
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for Version {}

/// Which of `apps` the stem names: by one of an app's forms (its name, or the last part of its bundle id, lowercased,
/// forms shorter than 3 characters dropped; a space in a form matches one separator or none), followed by the end, a
/// separator or a digit; every token of the rest a version, a plain number or one of `words`. Apple's apps are never
/// named. The version is the rest's first version token.
pub(crate) fn names_app(stem: &str, apps: &[InstalledApp], words: &Words) -> Named {
    let stem: Vec<char> = stem.to_lowercase().chars().collect();
    let mut named: Option<(usize, Option<Version>)> = None;
    for (index, app) in apps.iter().enumerate() {
        if app.bundle_id.folded().starts_with(APPLE) {
            continue;
        }
        let found =
            forms(app).iter().find_map(|form| rest_after(&stem, form).and_then(|rest| versions_of(rest, words)));
        let Some(versions) = found else { continue };
        if named.is_some() {
            return Named::Several;
        }
        named = Some((index, versions.into_iter().next()));
    }
    named.map_or(Named::None, |(app, version)| Named::One { app, version })
}

/// The app's forms, lowercased, the longest first, none shorter than `SHORTEST_FORM`.
fn forms(app: &InstalledApp) -> Vec<String> {
    let last = app.bundle_id.folded().rsplit('.').next().unwrap_or_default().to_owned();
    let mut forms: Vec<String> =
        [app.name.to_lowercase(), last].into_iter().filter(|form| form.chars().count() >= SHORTEST_FORM).collect();
    forms.sort_by_key(|form| std::cmp::Reverse(form.chars().count()));
    forms.dedup();
    forms
}

/// What follows `form` at the start of `stem`, past a separator: `None` when the stem does not start with the form,
/// or the form runs on into a letter.
fn rest_after<'a>(stem: &'a [char], form: &str) -> Option<&'a [char]> {
    let mut at = 0;
    for wanted in form.chars() {
        let next = stem.get(at).copied();
        if wanted == ' ' {
            if next.is_some_and(is_separator) {
                at += 1;
            }
        } else if next == Some(wanted) {
            at += 1;
        } else {
            return None;
        }
    }
    let rest = stem.get(at..)?;
    match rest.first() {
        None => Some(rest),
        Some(&next) if is_separator(next) || next == '.' => rest.get(1..),
        Some(next) if next.is_ascii_digit() => Some(rest),
        Some(_) => None,
    }
}

/// The versions of `rest`, in order, when every token of it is a version, a plain number or one of `words`; `None`
/// when one is not.
fn versions_of(rest: &[char], words: &Words) -> Option<Vec<Version>> {
    let text: String = rest.iter().collect();
    let mut versions = Vec::new();
    for token in text.split(is_separator).filter(|token| !token.is_empty()) {
        if let Some(version) = Version::parse(token) {
            versions.push(version);
        } else if !token.bytes().all(|byte| byte.is_ascii_digit()) && !words.holds(token) {
            return None;
        }
    }
    Some(versions)
}

/// What separates the words of a file name.
fn is_separator(character: char) -> bool {
    matches!(character, ' ' | '-' | '_')
}

#[cfg(test)]
mod tests {
    use fresher_core::InstalledApp;
    use fresher_testing::installed_app;

    use crate::installers::Words;
    use crate::name::{Named, names_app};

    fn app(id: &str, name: &str) -> InstalledApp {
        installed_app(id, name, "1.0").unwrap()
    }

    fn named(stem: &str, apps: &[InstalledApp]) -> Named {
        names_app(stem, apps, &Words::shipped().unwrap())
    }

    mod negative {
        use super::{app, named};
        use crate::name::{Named, Version};

        #[test]
        fn a_name_with_another_word_names_no_app() {
            let notes = [app("com.example.Notes", "Notes")];

            assert_eq!(named("notes-backup", &notes), Named::None);
            assert_eq!(named("install", &notes), Named::None);
        }

        #[test]
        fn apples_own_apps_are_never_named() {
            assert_eq!(named("Safari 18.0", &[app("com.apple.Safari", "Safari")]), Named::None);
        }

        #[test]
        fn a_form_shorter_than_three_letters_is_dropped() {
            assert_eq!(named("go-1.2", &[app("com.example.Go", "Go")]), Named::None);
        }

        #[test]
        fn a_name_running_on_into_letters_is_not_named() {
            assert_eq!(named("Firefoxsync", &[app("org.mozilla.firefox", "Firefox")]), Named::None);
        }

        #[test]
        fn a_copy_mark_names_no_app() {
            let firefox = [app("org.mozilla.firefox", "Firefox")];

            assert_eq!(named("Firefox (1)", &firefox), Named::None);
            assert_eq!(named("Firefox 133.0 (1)", &firefox), Named::None);
        }

        #[test]
        fn two_apps_named_is_several() {
            let apps = [app("com.example.Whale", "Docker"), app("com.example.docker", "Containers")];

            assert_eq!(named("docker", &apps), Named::Several);
        }

        #[test]
        fn a_version_with_letters_is_not_comparable() {
            assert!(Version::parse("1.0b2").is_none());
            assert!(Version::of_app("1.0b2").is_none());
            assert!(Version::parse("133").is_none());
            assert!(Version::parse("1..2").is_none());
            assert!(Version::parse("1.").is_none());
            assert!(Version::of_app("").is_none());
            assert_eq!(named("Firefox 1.0b2", &[app("org.mozilla.firefox", "Firefox")]), Named::None);
        }
    }

    mod positive {
        use super::{app, named};
        use crate::name::{Named, Version};

        fn version(text: &str) -> Option<Version> {
            Version::parse(text)
        }

        #[test]
        fn the_designs_examples_are_named() {
            let apps = [
                app("org.mozilla.firefox", "Firefox"),
                app("com.docker.docker", "Docker"),
                app("com.google.Chrome", "Google Chrome"),
                app("com.example.Example", "Example"),
            ];

            assert_eq!(named("Firefox 133.0", &apps), Named::One { app: 0, version: version("133.0") });
            assert_eq!(named("docker", &apps), Named::One { app: 1, version: None });
            assert_eq!(named("GoogleChrome", &apps), Named::One { app: 2, version: None });
            assert_eq!(named("Google-Chrome", &apps), Named::One { app: 2, version: None });
            assert_eq!(named("Example-2.1-arm64", &apps), Named::One { app: 3, version: version("2.1") });
            assert_eq!(named("firefox-133.0-universal", &apps), Named::One { app: 0, version: version("133.0") });
            assert_eq!(named("Firefox133.0", &apps), Named::One { app: 0, version: version("133.0") });
            assert_eq!(named("Firefox Setup v133.0.1", &apps), Named::One { app: 0, version: version("133.0.1") });
        }

        #[test]
        fn the_last_part_of_the_bundle_id_names_the_app() {
            let apps = [app("org.mozilla.firefox", "Browser")];

            assert_eq!(named("firefox-133.0", &apps), Named::One { app: 0, version: version("133.0") });
        }

        #[test]
        fn versions_compare_by_parts() {
            let parse = |text: &str| Version::of_app(text).unwrap();

            assert!(parse("133.0") <= parse("133.0.1"));
            assert!(parse("2.10") > parse("2.9"));
            assert_eq!(parse("1"), parse("1.0"));
            assert_eq!(Version::parse("v2.1"), Version::parse("2.1.0"));
        }
    }
}
