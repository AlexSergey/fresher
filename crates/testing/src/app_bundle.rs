use std::fmt::Write as _;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, SystemTime};

use fresher_core::{BundleId, InstalledBy, Manager};
use serde::Deserialize;

use crate::facts_lens::StatedOwner;
use crate::fixture_tree::FixtureError;
use crate::tree_description::{resolve, time_at_age};

/// The managers an `installed-by` names, as `<manager>:<token>`.
const MANAGERS: [(&str, Manager); 1] = [("homebrew", Manager::Homebrew)];

/// An installed app written as a real bundle: `<path>/Contents/Info.plist`, an XML property list, and an empty
/// `Contents/MacOS/`. It has no executable, so it is unsigned and its team id reads as none.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct AppBundle {
    /// The bundle folder, relative to the tree's root; it ends in `.app`.
    pub path: PathBuf,
    /// `CFBundleIdentifier`.
    pub bundle_id: String,
    /// `CFBundleName`.
    pub name: String,
    /// `CFBundleShortVersionString`; absent, the bundle has no version string.
    pub version: Option<String>,
    /// `CFBundleVersion`.
    pub build: Option<String>,
    /// When the bundle was last changed, stated as an age (`"10d"`) the way a file's is; the bundle is written now.
    pub changed: Option<String>,
    /// When the system last saw the app used, stated as an age (`"240d"`): what Spotlight would answer.
    pub last_used: Option<String>,
    /// The package manager that installed it, as `<manager>:<token>` (`"homebrew:tool"`).
    pub installed_by: Option<String>,
    /// Who the bundle's own folder belongs to, stated as a `[[file]]`'s `owner` is.
    pub owner: Option<StatedOwner>,
}

impl AppBundle {
    /// The paths the bundle writes below its folder, relative to the tree's root.
    pub fn written(&self) -> [PathBuf; 2] {
        let contents = self.path.join("Contents");
        [contents.join("Info.plist"), contents.join("MacOS")]
    }

    /// Refuses a path that is not plain or does not end in `.app`, a bundle id the app would not read, and a control
    /// character in a text of the `Info.plist`, which an XML property list cannot hold.
    pub(crate) fn check(&self) -> Result<(), FixtureError> {
        if !is_plain(&self.path) {
            return Err(FixtureError::PathOutsideRoot { path: self.path.clone() });
        }
        if self.path.extension().is_none_or(|extension| extension != "app") {
            return Err(FixtureError::AppNotABundle { path: self.path.clone() });
        }
        BundleId::new(self.bundle_id.as_str())
            .map_err(|_| FixtureError::InvalidBundleId { path: self.path.clone(), id: self.bundle_id.clone() })?;
        let texts = [Some(&self.name), self.version.as_ref(), self.build.as_ref()];
        if texts.into_iter().flatten().any(|text| text.chars().any(char::is_control)) {
            return Err(FixtureError::AppTextControl { path: self.path.clone() });
        }
        let far = SystemTime::UNIX_EPOCH + Duration::from_secs(1 << 40);
        if self.last_used.as_deref().is_some_and(|age| time_at_age(age, far).is_none()) {
            return Err(FixtureError::AppLastUsed { path: self.path.clone() });
        }
        self.manager()?;
        Ok(())
    }

    /// The stated package manager: `None` when none is stated; refused when it is not `<manager>:<token>` of a known
    /// manager and a token it accepts.
    pub fn manager(&self) -> Result<Option<InstalledBy>, FixtureError> {
        let Some(text) = &self.installed_by else { return Ok(None) };
        parse_installed_by(text)
            .map(Some)
            .ok_or_else(|| FixtureError::AppInstalledBy { path: self.path.clone(), value: text.clone() })
    }

    /// Writes the bundle into the existing `folder`, through no link; built again, it rewrites its `Info.plist`.
    pub(crate) fn write_at(&self, folder: &Path) -> Result<(), FixtureError> {
        let [plist, executables] = self.written();
        let executables = resolve(folder, &executables)?;
        fs::create_dir_all(&executables).map_err(|source| FixtureError::Create { path: executables, source })?;
        let plist = resolve(folder, &plist)?;
        fs::write(&plist, self.info_plist()).map_err(|source| FixtureError::Create { path: plist, source })
    }

    /// The `Info.plist` text: a fixed template, every value escaped.
    fn info_plist(&self) -> String {
        let mut entries = vec![
            ("CFBundleIdentifier", self.bundle_id.as_str()),
            ("CFBundleName", self.name.as_str()),
            ("CFBundlePackageType", "APPL"),
        ];
        if let Some(version) = &self.version {
            entries.push(("CFBundleShortVersionString", version));
        }
        if let Some(build) = &self.build {
            entries.push(("CFBundleVersion", build));
        }
        let mut body = String::new();
        for (key, value) in entries {
            let _ = write!(body, "\t<key>{key}</key>\n\t<string>{}</string>\n", escape(value));
        }
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
             <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
             <plist version=\"1.0\">\n<dict>\n{body}</dict>\n</plist>\n"
        )
    }
}

/// `<manager>:<token>` read as the manager that installed an app; `None` for an unknown manager or a refused token.
pub fn parse_installed_by(text: &str) -> Option<InstalledBy> {
    let (name, token) = text.split_once(':')?;
    let (_, manager) = MANAGERS.iter().find(|(known, _)| *known == name)?;
    InstalledBy::new(*manager, token).ok()
}

/// The manager and token as `<manager>:<token>`, the form [`parse_installed_by`] reads.
pub fn installed_by_text(installed_by: &InstalledBy) -> String {
    let name = MANAGERS.iter().find(|(_, manager)| *manager == installed_by.manager()).map_or("", |(name, _)| name);
    format!("{name}:{}", installed_by.token())
}

/// `true` for a non-empty relative path of plain names only.
fn is_plain(path: &Path) -> bool {
    !path.as_os_str().is_empty() && path.components().all(|component| matches!(component, Component::Normal(_)))
}

/// The text with XML's special characters as entities.
fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for char in text.chars() {
        match char {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            other => escaped.push(other),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::AppBundle;

    fn app(path: &str, id: &str) -> AppBundle {
        AppBundle {
            path: PathBuf::from(path),
            bundle_id: id.into(),
            name: "A & <B>".into(),
            version: None,
            build: None,
            changed: None,
            last_used: None,
            installed_by: None,
            owner: None,
        }
    }

    mod negative {
        use super::app;
        use crate::FixtureError;

        #[test]
        fn a_path_not_ending_in_app_is_refused() {
            for path in ["Applications/Tool", "Applications/Tool.APP.bak", "Applications/Tool.app/x.txt"] {
                assert!(matches!(app(path, "com.example.Tool").check(), Err(FixtureError::AppNotABundle { .. })));
            }
        }

        #[test]
        fn a_path_that_is_not_plain_is_refused() {
            for path in ["../Tool.app", "/Applications/Tool.app", "Applications/../Tool.app", ""] {
                let error = app(path, "com.example.Tool").check().unwrap_err();

                assert!(matches!(error, FixtureError::PathOutsideRoot { .. }), "{path:?}");
            }
        }

        #[test]
        fn a_control_character_in_a_text_of_the_plist_is_refused() {
            for text in ["Tab\tName", "Line\nbreak", "Null\0", "Bell\u{7}"] {
                let named = crate::AppBundle { name: text.into(), ..app("Applications/Tool.app", "com.example.Tool") };
                let versioned =
                    crate::AppBundle { version: Some(text.into()), ..app("Applications/Tool.app", "com.example.Tool") };
                let built =
                    crate::AppBundle { build: Some(text.into()), ..app("Applications/Tool.app", "com.example.Tool") };

                for bundle in [named, versioned, built] {
                    assert!(matches!(bundle.check(), Err(FixtureError::AppTextControl { .. })), "{text:?}");
                }
            }
        }

        #[test]
        fn a_last_use_that_is_not_an_age_is_refused() {
            for age in ["240", "d", "yesterday", "240 d"] {
                let bundle = crate::AppBundle {
                    last_used: Some(age.into()),
                    ..app("Applications/Tool.app", "com.example.Tool")
                };

                assert!(matches!(bundle.check(), Err(FixtureError::AppLastUsed { .. })), "{age:?}");
            }
        }

        #[test]
        fn an_installed_by_of_no_known_manager_or_a_refused_token_is_refused() {
            for value in ["homebrew", "homebrew:", "macports:tool", "Homebrew:tool", "homebrew:Tool", "homebrew:a b"] {
                let bundle = crate::AppBundle {
                    installed_by: Some(value.into()),
                    ..app("Applications/Tool.app", "com.example.Tool")
                };

                assert!(matches!(bundle.check(), Err(FixtureError::AppInstalledBy { .. })), "{value:?}");
                assert_eq!(crate::parse_installed_by(value), None, "{value:?}");
            }
        }

        #[test]
        fn a_bundle_id_the_app_would_not_read_is_refused() {
            for id in ["not an id", "", ".com.example", "com..example"] {
                let error = app("Applications/Tool.app", id).check().unwrap_err();

                assert!(matches!(error, FixtureError::InvalidBundleId { .. }), "{id:?}");
            }
        }
    }

    mod positive {
        use fresher_core::{InstalledBy, Manager};

        use super::app;

        #[test]
        fn an_installed_by_reads_its_manager_and_token_and_writes_back_the_same() {
            let bundle = crate::AppBundle {
                installed_by: Some("homebrew:tool@2".into()),
                last_used: Some("240d".into()),
                ..app("Applications/Tool.app", "com.example.Tool")
            };

            bundle.check().unwrap();
            let manager = bundle.manager().unwrap().unwrap();

            assert_eq!(manager, InstalledBy::new(Manager::Homebrew, "tool@2").unwrap());
            assert_eq!(crate::installed_by_text(&manager), "homebrew:tool@2");
        }

        #[test]
        fn the_plist_escapes_its_values_and_omits_a_missing_version() {
            let plist = app("Applications/Tool.app", "com.example.Tool").info_plist();

            assert!(plist.contains("<string>A &amp; &lt;B&gt;</string>"));
            assert!(plist.contains("<key>CFBundlePackageType</key>\n\t<string>APPL</string>"));
            assert!(!plist.contains("CFBundleShortVersionString") && !plist.contains("CFBundleVersion"));
        }

        #[test]
        fn writing_twice_rewrites_the_plist() {
            let folder = tempfile::tempdir().unwrap();
            let mut tool = app("Applications/Tool.app", "com.example.Tool");
            tool.write_at(folder.path()).unwrap();
            tool.version = Some("2.0".into());

            tool.write_at(folder.path()).unwrap();

            let plist = folder.path().join("Applications/Tool.app/Contents/Info.plist");
            assert!(std::fs::read_to_string(plist).unwrap().contains("<string>2.0</string>"));
            assert!(folder.path().join("Applications/Tool.app/Contents/MacOS").is_dir());
        }
    }
}
