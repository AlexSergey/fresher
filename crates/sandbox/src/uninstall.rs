use std::path::{Path, PathBuf};

use fresher_core::{NotOffered, UninstallChoice};
use serde::Deserialize;
use serde_json::Value;

use crate::case::is_plain;
use crate::{CaseError, CaseKind};
use fresher_testing::TreeDescription;

/// What the Apps screen must show of one of a case's apps (`[case.uninstall]`), and what its uninstall does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UninstallExpectation {
    app: PathBuf,
    expected: Uninstall,
}

/// How the Apps screen shows the app.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Uninstall {
    /// Not in the list.
    NotListed,
    /// In the list, never offered, for this reason.
    NotOffered(NotOffered),
    /// Uninstalled with `choice`: `trashed` are in the sandbox's Trash afterwards (the bundle first), `kept` in place.
    Chosen { choice: UninstallChoice, trashed: Vec<PathBuf>, kept: Vec<PathBuf> },
}

/// `[case.uninstall]` as written; every value is checked by [`UninstallExpectation::from_raw`].
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawUninstall {
    app: PathBuf,
    listed: Option<bool>,
    offered: Option<bool>,
    reason: Option<String>,
    choice: Option<String>,
    trashed: Option<Vec<PathBuf>>,
    #[serde(default)]
    kept: Vec<PathBuf>,
}

impl UninstallExpectation {
    /// Checks the table against the case: `app` is one of its `[[case.app]]`; exactly one of `listed = false`,
    /// `offered = false` with a `reason`, or a `choice` with `trashed` (the bundle first) and `kept`, every path one
    /// the case writes and in one list only; a choice on a positive case, the two others on a negative one.
    pub(crate) fn from_raw(
        case: &str,
        kind: CaseKind,
        tree: &TreeDescription,
        raw: RawUninstall,
    ) -> Result<Self, CaseError> {
        if !tree.apps().iter().any(|app| app.path == raw.app) {
            return Err(CaseError::UninstallUnknownApp { case: case.into(), app: raw.app });
        }
        let forms = || CaseError::UninstallForms { case: case.into() };
        let lists = raw.trashed.is_some() || !raw.kept.is_empty();
        let expected = match (raw.listed, raw.offered, raw.choice) {
            (Some(false), None, None) if raw.reason.is_none() => Uninstall::NotListed,
            (None, Some(false), None) => {
                let reason = raw.reason.ok_or_else(forms)?;
                let code = NotOffered::ALL.into_iter().find(|code| code.code() == reason);
                Uninstall::NotOffered(code.ok_or_else(|| CaseError::UninstallReason { case: case.into(), reason })?)
            }
            (None, None, Some(choice)) if raw.reason.is_none() => {
                let named = serde_json::from_value(Value::String(choice.clone()));
                let choice = named.map_err(|_| CaseError::UninstallChoice { case: case.into(), choice })?;
                let trashed = raw.trashed.ok_or_else(|| CaseError::UninstallTrashed { case: case.into() })?;
                if trashed.first() != Some(&raw.app) {
                    return Err(CaseError::UninstallTrashed { case: case.into() });
                }
                check_paths(case, tree, &trashed, &raw.kept)?;
                Uninstall::Chosen { choice, trashed, kept: raw.kept }
            }
            _ => return Err(forms()),
        };
        let chosen = matches!(expected, Uninstall::Chosen { .. });
        if !chosen && lists {
            return Err(forms());
        }
        match (kind, chosen) {
            (CaseKind::Positive, true) | (CaseKind::Negative, false) => {}
            (CaseKind::Positive | CaseKind::Negative | CaseKind::Scenery, _) => {
                return Err(CaseError::UninstallOnTheWrongKind { case: case.into() });
            }
        }
        Ok(Self { app: raw.app, expected })
    }

    /// The app's bundle, relative to the sandbox folder.
    pub fn app(&self) -> &Path {
        &self.app
    }

    /// Whether the app is uninstalled with a choice.
    pub(crate) const fn is_chosen(&self) -> bool {
        matches!(self.expected, Uninstall::Chosen { .. })
    }

    /// How the Apps screen shows it, and what its uninstall does.
    pub const fn expected(&self) -> &Uninstall {
        &self.expected
    }
}

/// Every path is plain, one of the case's paths or their folders, and in one of the two lists only.
fn check_paths(case: &str, tree: &TreeDescription, trashed: &[PathBuf], kept: &[PathBuf]) -> Result<(), CaseError> {
    for path in trashed.iter().chain(kept) {
        let own = tree.paths().any(|written| written.ancestors().any(|folder| folder == path));
        if !is_plain(path) || !own {
            return Err(CaseError::UninstallPathNotInCase { case: case.into(), path: path.clone() });
        }
    }
    let mut seen: Vec<&PathBuf> = Vec::new();
    for path in trashed.iter().chain(kept) {
        if seen.contains(&path) {
            return Err(CaseError::UninstallPathTwice { case: case.into(), path: path.clone() });
        }
        seen.push(path);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    /// A one-case file of `kind` installing `Applications/Keeper.app` with a cache and a setting, and this
    /// `[case.uninstall]` body.
    fn with_uninstall(kind: &str, body: &str) -> String {
        format!(
            "feature = \"uninstall\"\nmodule = \"trial\"\n[[case]]\nname = \"c\"\nkind = \"{kind}\"\n\
             [[case.app]]\npath = \"Applications/Keeper.app\"\nbundle-id = \"com.example.Keeper\"\nname = \"Keeper\"\n\
             [[case.file]]\npath = \"home/Library/Caches/com.example.Keeper/a.bin\"\n\
             [[case.file]]\npath = \"home/Library/Preferences/com.example.Keeper.plist\"\n\
             [case.uninstall]\napp = \"Applications/Keeper.app\"\n{body}"
        )
    }

    const CACHE: &str = "home/Library/Caches/com.example.Keeper";
    const SETTINGS: &str = "home/Library/Preferences/com.example.Keeper.plist";

    mod negative {
        use super::{CACHE, SETTINGS, with_uninstall};
        use crate::{CaseError, CaseFile};

        fn refused(text: &str) -> CaseError {
            CaseFile::parse(text).unwrap_err()
        }

        #[test]
        fn an_app_the_case_does_not_declare_is_refused() {
            let text = with_uninstall("negative", "listed = false\n")
                .replace("app = \"Applications/Keeper.app\"\nlisted", "app = \"Applications/Other.app\"\nlisted");

            let error = refused(&text);

            assert!(matches!(&error, CaseError::UninstallUnknownApp { case, .. } if case == "c"), "{error}");
        }

        #[test]
        fn two_forms_at_once_or_none_are_refused() {
            for (kind, body) in [
                ("negative", "listed = false\noffered = false\nreason = \"open\"\n"),
                ("positive", "listed = false\nchoice = \"keep-data\"\ntrashed = [\"Applications/Keeper.app\"]\n"),
                ("negative", "offered = false\n"),
                ("negative", "listed = true\n"),
                ("negative", "offered = true\nreason = \"open\"\n"),
                ("negative", "listed = false\nreason = \"open\"\n"),
                ("negative", "listed = false\nkept = [\"home/Library/Caches/com.example.Keeper\"]\n"),
                ("negative", ""),
            ] {
                let error = refused(&with_uninstall(kind, body));

                assert!(matches!(&error, CaseError::UninstallForms { case } if case == "c"), "{body}: {error}");
            }
        }

        #[test]
        fn an_unknown_reason_or_choice_is_refused() {
            let reason = refused(&with_uninstall("negative", "offered = false\nreason = \"busy\"\n"));
            let choice =
                refused(&with_uninstall("positive", "choice = \"all\"\ntrashed = [\"Applications/Keeper.app\"]\n"));

            assert!(matches!(&reason, CaseError::UninstallReason { reason, .. } if reason == "busy"), "{reason}");
            assert!(matches!(&choice, CaseError::UninstallChoice { choice, .. } if choice == "all"), "{choice}");
        }

        #[test]
        fn a_choice_without_trashed_or_not_bundle_first_is_refused() {
            for body in [
                "choice = \"keep-data\"\n".to_owned(),
                "choice = \"keep-data\"\ntrashed = []\n".to_owned(),
                format!("choice = \"keep-data\"\ntrashed = [\"{CACHE}\", \"Applications/Keeper.app\"]\n"),
            ] {
                let error = refused(&with_uninstall("positive", &body));

                assert!(matches!(&error, CaseError::UninstallTrashed { case } if case == "c"), "{body}: {error}");
            }
        }

        #[test]
        fn a_path_the_case_does_not_write_is_refused() {
            for path in
                ["home/Library/Caches/com.example.Other", "../Keeper.app", "home/Library/Caches/com.example.Keeper/b"]
            {
                let body =
                    format!("choice = \"keep-data\"\ntrashed = [\"Applications/Keeper.app\"]\nkept = [\"{path}\"]\n");

                let error = refused(&with_uninstall("positive", &body));

                assert!(matches!(&error, CaseError::UninstallPathNotInCase { .. }), "{path}: {error}");
            }
        }

        #[test]
        fn a_path_in_both_lists_is_refused() {
            let body = format!(
                "choice = \"everything\"\ntrashed = [\"Applications/Keeper.app\", \"{SETTINGS}\"]\n\
                 kept = [\"{SETTINGS}\"]\n"
            );

            let error = refused(&with_uninstall("positive", &body));

            assert!(
                matches!(&error, CaseError::UninstallPathTwice { path, .. } if path.as_os_str() == SETTINGS),
                "{error}"
            );
        }

        #[test]
        fn a_choice_on_a_negative_or_a_refusal_on_a_positive_is_refused() {
            let chosen = refused(&with_uninstall(
                "negative",
                "choice = \"keep-data\"\ntrashed = [\"Applications/Keeper.app\"]\n",
            ));
            let refusal = refused(&with_uninstall("positive", "listed = false\n"));

            assert!(matches!(&chosen, CaseError::UninstallOnTheWrongKind { .. }), "{chosen}");
            assert!(matches!(&refusal, CaseError::UninstallOnTheWrongKind { .. }), "{refusal}");
        }

        #[test]
        fn another_case_writing_in_the_uninstalled_bundle_is_refused() {
            let body = "choice = \"keep-data\"\ntrashed = [\"Applications/Keeper.app\"]\n";
            let other = "[[case]]\nname = \"inside\"\nkind = \"positive\"\n[[case.file]]\n\
                         path = \"Applications/Keeper.app/Contents/Resources/extra.bin\"\n[case.expect]\n\
                         item = \"Applications/Keeper.app/Contents/Resources/extra.bin\"\ncategory = \"trial.caches\"\n\
                         danger = \"safe\"\nafter-cleanup = \"in-trash\"\n";

            let error = refused(&format!("{}{other}", with_uninstall("positive", body)));

            assert!(
                matches!(&error, CaseError::UninstallBundleShared { case, other, .. } if case == "c" && other == "inside"),
                "{error}"
            );
        }

        #[test]
        fn an_unknown_uninstall_field_is_refused() {
            let error = refused(&with_uninstall("negative", "listed = false\nshown = true\n"));

            assert!(matches!(error, CaseError::Parse { .. }), "{error}");
        }
    }

    mod positive {
        use std::path::PathBuf;

        use fresher_core::{NotOffered, UninstallChoice};

        use super::{CACHE, SETTINGS, with_uninstall};
        use crate::{CaseFile, Uninstall};

        fn parsed(kind: &str, body: &str) -> Uninstall {
            let file = CaseFile::parse(&with_uninstall(kind, body)).unwrap();
            file.cases()[0].uninstall().unwrap().expected().clone()
        }

        #[test]
        fn not_listed_and_not_offered_read_on_a_negative() {
            assert_eq!(parsed("negative", "listed = false\n"), Uninstall::NotListed);
            for reason in NotOffered::ALL {
                let body = format!("offered = false\nreason = \"{}\"\n", reason.code());

                assert_eq!(parsed("negative", &body), Uninstall::NotOffered(reason));
            }
        }

        #[test]
        fn a_choice_reads_its_trashed_and_kept_paths_on_a_positive_without_expect() {
            let body = format!(
                "choice = \"keep-data\"\ntrashed = [\"Applications/Keeper.app\", \"{CACHE}\"]\n\
                 kept = [\"{SETTINGS}\"]\n"
            );

            let file = CaseFile::parse(&with_uninstall("positive", &body)).unwrap();

            let uninstall = file.cases()[0].uninstall().unwrap();
            assert_eq!(uninstall.app(), PathBuf::from("Applications/Keeper.app"));
            assert_eq!(
                uninstall.expected(),
                &Uninstall::Chosen {
                    choice: UninstallChoice::KeepData,
                    trashed: vec!["Applications/Keeper.app".into(), CACHE.into()],
                    kept: vec![SETTINGS.into()],
                }
            );
            assert!(file.cases()[0].expect().is_none());
        }

        #[test]
        fn a_case_without_the_table_has_none() {
            let file = CaseFile::parse(
                &with_uninstall("negative", "listed = false\n")
                    .replace("[case.uninstall]\napp = \"Applications/Keeper.app\"\nlisted = false\n", ""),
            )
            .unwrap();

            assert!(file.cases()[0].uninstall().is_none());
        }
    }
}
