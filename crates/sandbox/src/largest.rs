use std::path::{Path, PathBuf};

use fresher_core::{ItemKind, Withheld};
use serde::Deserialize;
use serde_json::Value;

use crate::CaseError;
use crate::case::{is_bundle_name, is_plain};
use fresher_testing::TreeDescription;

/// What the Largest screen must show of a case's files (`[case.largest]`): a row offered, nothing listed, a Browse row
/// withheld for a reason, a guarded folder not seen, or a folder listed or not listed among the folders left alone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LargestExpectation {
    shown: Shown,
    at: PathBuf,
    kind: Option<ItemKind>,
    withheld: Option<Withheld>,
    unpacked: Option<String>,
    project: Option<bool>,
}

/// How the Largest screen shows the case's `at`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shown {
    /// A Files row at `at` (a Folders row for a folder) that can be chosen.
    Offered,
    /// No Files row at or below `at`.
    Absent,
    /// The Browse row at `at` carries the case's `withheld` reason.
    Withheld,
    /// No Files row at or below `at`, and the last scan did not enter the guarded folder `at` is in.
    NotSeen,
    /// The folder `at` is a row of the Folders list (the folders left alone), as a project when the case says.
    Listed,
    /// The folder `at` is no row of the Folders list.
    Unlisted,
}

impl Shown {
    const ALL: [(&'static str, Self); 6] = [
        ("offered", Self::Offered),
        ("absent", Self::Absent),
        ("withheld", Self::Withheld),
        ("not-seen", Self::NotSeen),
        ("listed", Self::Listed),
        ("unlisted", Self::Unlisted),
    ];
}

/// `[case.largest]` as written; every value is checked by [`LargestExpectation::from_raw`].
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawLargest {
    shown: String,
    at: Option<PathBuf>,
    kind: Option<String>,
    withheld: Option<String>,
    unpacked: Option<String>,
    project: Option<bool>,
}

impl RawLargest {
    /// Whether the table says a folder is listed on Folders: a positive case, which cleans nothing.
    pub(crate) fn lists_a_folder(&self) -> bool {
        self.shown == "listed"
    }
}

impl LargestExpectation {
    /// Checks the table against the case's files: `at` is one of the case's paths or their folders (by default its
    /// first file, which only `offered` and `absent` may leave out); `withheld` with `shown = "withheld"` and only
    /// with it; `kind` and `unpacked` with `offered` only; `project` with `listed` only; `listed` and `unlisted` at a
    /// folder of the case.
    pub(crate) fn from_raw(case: &str, tree: &TreeDescription, raw: RawLargest) -> Result<Self, CaseError> {
        let shown = Shown::ALL
            .iter()
            .find(|(name, _)| *name == raw.shown)
            .map(|&(_, shown)| shown)
            .ok_or_else(|| CaseError::LargestShown { case: case.into(), shown: raw.shown.clone() })?;
        let withheld = match (shown, raw.withheld) {
            (Shown::Withheld, Some(reason)) => Some(
                withheld_named(&reason)
                    .ok_or_else(|| CaseError::LargestWithheldReason { case: case.into(), reason })?,
            ),
            (Shown::Withheld, None)
            | (Shown::Offered | Shown::Absent | Shown::NotSeen | Shown::Listed | Shown::Unlisted, Some(_)) => {
                return Err(CaseError::LargestWithheld { case: case.into() });
            }
            (Shown::Offered | Shown::Absent | Shown::NotSeen | Shown::Listed | Shown::Unlisted, None) => None,
        };
        if raw.project.is_some() && shown != Shown::Listed {
            return Err(CaseError::LargestProject { case: case.into() });
        }
        if shown != Shown::Offered {
            let field = match (&raw.kind, &raw.unpacked) {
                (Some(_), _) => Some("kind"),
                (None, Some(_)) => Some("unpacked"),
                (None, None) => None,
            };
            if let Some(field) = field {
                return Err(CaseError::LargestOfferedOnly { case: case.into(), field });
            }
        }
        let kind = raw
            .kind
            .map(|kind| kind_named(&kind).ok_or_else(|| CaseError::LargestKind { case: case.into(), kind }))
            .transpose()?;
        if let Some(name) = raw.unpacked.as_deref().filter(|name| !is_bundle_name(name)) {
            return Err(CaseError::LargestUnpacked { case: case.into(), name: name.into() });
        }
        let at = match (raw.at, shown) {
            (Some(at), _) => at,
            (None, Shown::Offered | Shown::Absent) => tree
                .files()
                .chain(tree.paths())
                .next()
                .map(Path::to_path_buf)
                .ok_or_else(|| CaseError::LargestWithoutAt { case: case.into() })?,
            (None, Shown::Withheld | Shown::NotSeen | Shown::Listed | Shown::Unlisted) => {
                return Err(CaseError::LargestWithoutAt { case: case.into() });
            }
        };
        let own = tree.paths().chain(tree.markers()).any(|path| path.ancestors().any(|folder| folder == at));
        if !is_plain(&at) || !own {
            return Err(CaseError::LargestAtOutsideTheCase { case: case.into(), at });
        }
        if matches!(shown, Shown::Listed | Shown::Unlisted) && tree.files().any(|file| file == at) {
            return Err(CaseError::LargestAtIsAFile { case: case.into(), at });
        }
        Ok(Self { shown, at, kind, withheld, unpacked: raw.unpacked, project: raw.project })
    }

    /// How the screen shows `at`.
    pub const fn shown(&self) -> Shown {
        self.shown
    }

    /// The path the expectation is about, relative to the sandbox folder.
    pub fn at(&self) -> &Path {
        &self.at
    }

    /// The kind an offered row is of, when the case says.
    pub const fn kind(&self) -> Option<ItemKind> {
        self.kind
    }

    /// Why the row cannot be chosen: given with `Shown::Withheld`, and only with it.
    pub const fn withheld(&self) -> Option<Withheld> {
        self.withheld
    }

    /// Whether a listed folder is a project, when the case says.
    pub const fn project(&self) -> Option<bool> {
        self.project
    }

    /// The folder an offered archive's row names beside it, when the case says.
    pub fn unpacked(&self) -> Option<&str> {
        self.unpacked.as_deref()
    }
}

/// The kind serialized as `name` (`disk-image`).
fn kind_named(name: &str) -> Option<ItemKind> {
    serde_json::from_value(Value::String(name.to_owned())).ok()
}

/// The reason serialized as `name` (`standard-folder`).
fn withheld_named(name: &str) -> Option<Withheld> {
    Withheld::ALL.into_iter().find(|reason| serde_json::to_value(reason).is_ok_and(|value| value == name))
}

#[cfg(test)]
mod tests {
    /// A one-case file: a negative or positive writing `home/VMs/Old.utm/disk.img` and `home/Movies/a.txt`, with
    /// this `[case.largest]` body.
    fn with_largest(kind: &str, body: &str) -> String {
        let expect = if kind == "positive" {
            "[case.expect]\nitem = \"home/VMs/Old.utm\"\ncategory = \"core.own-file\"\ndanger = \"high\"\n\
             after-cleanup = \"in-trash\"\n"
        } else {
            ""
        };
        format!(
            "feature = \"large-files\"\nmodule = \"trial\"\n[[case]]\nname = \"c\"\nkind = \"{kind}\"\n\
             [[case.file]]\npath = \"home/VMs/Old.utm/disk.img\"\n[[case.file]]\npath = \"home/Movies/a.txt\"\n\
             {expect}[case.largest]\n{body}"
        )
    }

    mod negative {
        use super::with_largest;
        use crate::{CaseError, CaseFile};

        fn refused(text: &str) -> CaseError {
            CaseFile::parse(text).unwrap_err()
        }

        #[test]
        fn an_unknown_shown_is_refused() {
            for shown in ["Listed", "Offered", ""] {
                let error = refused(&with_largest("negative", &format!("shown = \"{shown}\"\n")));

                assert!(
                    matches!(&error, CaseError::LargestShown { case, shown: found }
                    if case == "c" && found == shown),
                    "{shown}: {error}"
                );
                assert!(
                    error.to_string().contains("offered, absent, withheld, not-seen, listed or unlisted"),
                    "{error}"
                );
            }
        }

        #[test]
        fn a_project_without_listed_is_refused() {
            for shown in ["offered", "absent", "withheld", "not-seen", "unlisted"] {
                let withheld = if shown == "withheld" { "withheld = \"standard-folder\"\n" } else { "" };
                let body = format!("shown = \"{shown}\"\nat = \"home/Movies\"\n{withheld}project = true\n");

                let error = refused(&with_largest("negative", &body));

                assert!(matches!(&error, CaseError::LargestProject { case } if case == "c"), "{shown}: {error}");
            }
        }

        #[test]
        fn a_positive_that_only_unlists_a_folder_is_refused() {
            let text = with_largest("negative", "shown = \"unlisted\"\nat = \"home/Movies\"\n")
                .replace("kind = \"negative\"", "kind = \"positive\"");

            assert!(matches!(refused(&text), CaseError::PositiveWithoutExpect { .. }));
        }

        #[test]
        fn listed_or_unlisted_at_a_file_is_refused() {
            for shown in ["listed", "unlisted"] {
                let body = format!("shown = \"{shown}\"\nat = \"home/Movies/a.txt\"\n");

                let error = refused(&with_largest("negative", &body));

                assert!(matches!(&error, CaseError::LargestAtIsAFile { case, .. } if case == "c"), "{shown}: {error}");
            }
        }

        #[test]
        fn listed_or_unlisted_without_an_at_is_refused() {
            for shown in ["listed", "unlisted"] {
                let error = refused(&with_largest("negative", &format!("shown = \"{shown}\"\n")));

                assert!(matches!(&error, CaseError::LargestWithoutAt { case } if case == "c"), "{shown}: {error}");
            }
        }

        #[test]
        fn a_withheld_without_shown_withheld_is_refused() {
            for shown in ["offered", "absent", "not-seen", "listed", "unlisted"] {
                let body = format!("shown = \"{shown}\"\nat = \"home/Movies\"\nwithheld = \"standard-folder\"\n");

                let error = refused(&with_largest("negative", &body));

                assert!(matches!(&error, CaseError::LargestWithheld { case } if case == "c"), "{shown}: {error}");
            }
        }

        #[test]
        fn shown_withheld_without_a_reason_is_refused() {
            let error = refused(&with_largest("negative", "shown = \"withheld\"\nat = \"home/Movies\"\n"));

            assert!(matches!(error, CaseError::LargestWithheld { .. }), "{error}");
        }

        #[test]
        fn an_unknown_reason_is_refused() {
            let body = "shown = \"withheld\"\nat = \"home/Movies\"\nwithheld = \"Standard-Folder\"\n";

            let error = refused(&with_largest("negative", body));

            assert!(
                matches!(&error, CaseError::LargestWithheldReason { reason, .. } if reason == "Standard-Folder"),
                "{error}"
            );
        }

        #[test]
        fn a_kind_or_unpacked_without_offered_is_refused() {
            for (body, field) in [
                ("shown = \"absent\"\nkind = \"disk-image\"\n", "kind"),
                ("shown = \"absent\"\nunpacked = \"Photos\"\n", "unpacked"),
                ("shown = \"not-seen\"\nat = \"home/Movies\"\nkind = \"folder\"\n", "kind"),
            ] {
                let error = refused(&with_largest("negative", body));

                assert!(
                    matches!(&error, CaseError::LargestOfferedOnly { field: found, .. } if *found == field),
                    "{body}: {error}"
                );
            }
        }

        #[test]
        fn an_unknown_kind_is_refused() {
            let error = refused(&with_largest("positive", "shown = \"offered\"\nkind = \"vm\"\n"));

            assert!(matches!(&error, CaseError::LargestKind { kind, .. } if kind == "vm"), "{error}");
        }

        #[test]
        fn an_unpacked_that_is_not_one_name_is_refused() {
            for name in ["", "a/b", ".."] {
                let body = format!("shown = \"offered\"\nunpacked = \"{name}\"\n");

                let error = refused(&with_largest("positive", &body));

                assert!(matches!(&error, CaseError::LargestUnpacked { .. }), "{name:?}: {error}");
            }
        }

        #[test]
        fn withheld_or_not_seen_without_an_at_is_refused() {
            for body in ["shown = \"withheld\"\nwithheld = \"standard-folder\"\n", "shown = \"not-seen\"\n"] {
                let error = refused(&with_largest("negative", body));

                assert!(matches!(&error, CaseError::LargestWithoutAt { case } if case == "c"), "{body}: {error}");
            }
        }

        #[test]
        fn an_at_that_is_not_a_plain_path_of_the_case_is_refused() {
            for at in ["home/Music", "/home/Movies", "home/../home/Movies", "home/Movies/a.txt/b", ""] {
                let error = refused(&with_largest("negative", &format!("shown = \"absent\"\nat = \"{at}\"\n")));

                assert!(
                    matches!(&error, CaseError::LargestAtOutsideTheCase { case, .. } if case == "c"),
                    "{at:?}: {error}"
                );
            }
        }

        #[test]
        fn an_unknown_largest_field_is_refused() {
            let error = refused(&with_largest("negative", "shown = \"absent\"\ncolour = \"red\"\n"));

            assert!(matches!(error, CaseError::Parse { .. }), "{error}");
        }
    }

    mod positive {
        use std::path::Path;

        use fresher_core::{ItemKind, Withheld};

        use super::with_largest;
        use crate::{CaseFile, Shown};

        fn parsed(kind: &str, body: &str) -> crate::LargestExpectation {
            CaseFile::parse(&with_largest(kind, body)).unwrap().cases()[0].largest().unwrap().clone()
        }

        #[test]
        fn offered_and_absent_default_to_the_case_s_first_file() {
            for (kind, shown, expected) in
                [("positive", "offered", Shown::Offered), ("negative", "absent", Shown::Absent)]
            {
                let largest = parsed(kind, &format!("shown = \"{shown}\"\n"));

                assert_eq!(largest.shown(), expected);
                assert_eq!(largest.at(), Path::new("home/VMs/Old.utm/disk.img"));
                assert_eq!((largest.kind(), largest.withheld(), largest.unpacked()), (None, None, None));
            }
        }

        #[test]
        fn an_offered_row_reads_its_kind_and_unpacked_folder() {
            let body =
                "shown = \"offered\"\nat = \"home/VMs/Old.utm\"\nkind = \"virtual-machine\"\nunpacked = \"Old\"\n";

            let largest = parsed("positive", body);

            assert_eq!(largest.at(), Path::new("home/VMs/Old.utm"));
            assert_eq!(largest.kind(), Some(ItemKind::VirtualMachine));
            assert_eq!(largest.unpacked(), Some("Old"));
        }

        #[test]
        fn a_withheld_folder_reads_its_reason() {
            let largest =
                parsed("negative", "shown = \"withheld\"\nat = \"home/Movies\"\nwithheld = \"standard-folder\"\n");

            assert_eq!((largest.shown(), largest.withheld()), (Shown::Withheld, Some(Withheld::StandardFolder)));
        }

        #[test]
        fn every_reason_reads_by_its_serialized_name() {
            for reason in Withheld::ALL {
                let name = serde_json::to_value(reason).unwrap();
                let body = format!("shown = \"withheld\"\nat = \"home/Movies\"\nwithheld = {name}\n");

                assert_eq!(parsed("negative", &body).withheld(), Some(reason));
            }
        }

        #[test]
        fn not_seen_reads_its_folder() {
            let largest = parsed("negative", "shown = \"not-seen\"\nat = \"home/Movies\"\n");

            assert_eq!((largest.shown(), largest.at()), (Shown::NotSeen, Path::new("home/Movies")));
        }

        #[test]
        fn listed_and_unlisted_are_read() {
            for (shown, expected) in [("listed", Shown::Listed), ("unlisted", Shown::Unlisted)] {
                let largest = parsed("negative", &format!("shown = \"{shown}\"\nat = \"home/Movies\"\n"));

                assert_eq!((largest.shown(), largest.at()), (expected, Path::new("home/Movies")));
                assert_eq!((largest.project(), largest.withheld(), largest.kind()), (None, None, None));
            }
        }

        #[test]
        fn a_positive_that_lists_a_folder_needs_no_expect() {
            let text = with_largest("negative", "shown = \"listed\"\nat = \"home/Movies\"\n")
                .replace("kind = \"negative\"", "kind = \"positive\"");

            let file = CaseFile::parse(&text).unwrap();

            assert!(file.cases()[0].expect().is_none());
            assert_eq!(file.cases()[0].largest().unwrap().shown(), Shown::Listed);
        }

        #[test]
        fn a_listed_project_is_read() {
            for (written, project) in [("true", true), ("false", false)] {
                let body = format!("shown = \"listed\"\nat = \"home/VMs\"\nproject = {written}\n");

                assert_eq!(parsed("negative", &body).project(), Some(project));
            }
        }

        #[test]
        fn withheld_still_names_a_reason() {
            let largest =
                parsed("negative", "shown = \"withheld\"\nat = \"home/VMs/Old.utm\"\nwithheld = \"standard-folder\"\n");

            assert_eq!((largest.shown(), largest.withheld()), (Shown::Withheld, Some(Withheld::StandardFolder)));
            assert_eq!(largest.project(), None);
        }

        #[test]
        fn a_case_without_the_table_has_none() {
            let text =
                with_largest("negative", "shown = \"absent\"\n").replace("[case.largest]\nshown = \"absent\"\n", "");

            assert!(CaseFile::parse(&text).unwrap().cases()[0].largest().is_none());
        }
    }
}
