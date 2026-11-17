use std::fmt::Write as _;
use std::path::Path;

use fresher_core::{DangerLevel, SkipCode};

use crate::{AfterCleanup, Case, CaseFile, CaseKind, Expectation};

/// `EXPECTED.md`: per case file, one checkbox per case, negatives first, each with one plain sentence; then the
/// command that opens the app on the sandbox at `sandbox`.
pub(crate) fn checklist(files: &[CaseFile], sandbox: &Path) -> String {
    let mut text = String::from(
        "# What the sandbox must show\n\nOpen the app on this sandbox, scan, clean everything it offers, and tick each case.\n",
    );
    for file in files {
        let _ = write!(text, "\n## {}\n\nFrom `fixtures/sandboxes/{}.toml`.\n\n", file.feature(), file.feature());
        let ordered = [CaseKind::Negative, CaseKind::Positive]
            .into_iter()
            .flat_map(|kind| file.cases().iter().filter(move |case| case.kind() == kind));
        for case in ordered {
            let _ = writeln!(text, "- [ ] **{}**: {}.", case.name(), sentence(case));
        }
    }
    let _ = write!(text, "\n## Open the app on it\n\n```sh\nnpm run dev:sandbox -- {}\n```\n", sandbox.display());
    text
}

/// What must happen to the case, in one sentence.
fn sentence(case: &Case) -> String {
    match case.expect() {
        None => format!("never offered: {}; still there after the cleanup", top_paths(case)),
        Some(expect) => format!(
            "offered in *{}* ({}): `{}`; {}",
            expect.category().as_str(),
            danger_name(expect.danger()),
            expect.item().display(),
            after(expect)
        ),
    }
}

/// The case's paths that are not below another of its paths, each in backticks.
fn top_paths(case: &Case) -> String {
    let paths: Vec<&Path> = case.tree().paths().collect();
    let top: Vec<String> = paths
        .iter()
        .filter(|path| !paths.iter().any(|other| other != *path && path.starts_with(other)))
        .map(|path| format!("`{}`", path.display()))
        .collect();
    top.join(", ")
}

fn after(expect: &Expectation) -> String {
    match expect.after() {
        AfterCleanup::InTrash => "in the sandbox's Trash after the cleanup".into(),
        AfterCleanup::Emptied => "kept and emptied by the cleanup, its contents in the sandbox's Trash".into(),
        AfterCleanup::Skipped(code) => format!("skipped by the cleanup (`{}`), still there", code_name(code)),
    }
}

/// The danger level's name, as the app shows it.
const fn danger_name(danger: DangerLevel) -> &'static str {
    match danger {
        DangerLevel::Safe => "Safe",
        DangerLevel::Low => "Low",
        DangerLevel::Medium => "Medium",
        DangerLevel::High => "High",
    }
}

/// The skip code's name in a case file's `reason` (`in-use`).
const fn code_name(code: SkipCode) -> &'static str {
    match code {
        SkipCode::NotInSandbox => "not-in-sandbox",
        SkipCode::OutsideRoots => "outside-roots",
        SkipCode::Protected => "protected",
        SkipCode::WholeFolder => "whole-folder",
        SkipCode::Ignored => "ignored",
        SkipCode::ThroughLink => "through-link",
        SkipCode::Vanished => "vanished",
        SkipCode::OtherVolume => "other-volume",
        SkipCode::Replaced => "replaced",
        SkipCode::KindChanged => "kind-changed",
        SkipCode::NotRemovable => "not-removable",
        SkipCode::MissingCapability => "missing-capability",
        SkipCode::InUse => "in-use",
        SkipCode::Grown => "grown",
        SkipCode::NoTrash => "no-trash",
        SkipCode::Covered => "covered",
        SkipCode::NotTheSubject => "not-the-subject",
        SkipCode::ForcedTool => "forced-tool",
        SkipCode::CheckFailed => "check-failed",
    }
}

#[cfg(test)]
mod tests {
    const CASES: &str = r#"
feature = "trial"

[[case]]
name = "a look-alike is never offered"
kind = "negative"
[[case.dir]]
path = "home/work/fresher-trial-cache2"
[[case.file]]
path = "home/work/fresher-trial-cache2/look-alike.bin"
[[case.file]]
path = "home/other.bin"

[[case]]
name = "a cache goes to the Trash"
kind = "positive"
[[case.file]]
path = "home/work/fresher-trial-cache/blob.bin"
[case.expect]
item = "home/work/fresher-trial-cache"
category = "trial.caches"
danger = "safe"
after-cleanup = "in-trash"

[[case]]
name = "a busy cache is skipped"
kind = "positive"
[[case.file]]
path = "home/busy/fresher-trial-cache/blob.bin"
[case.expect]
item = "home/busy/fresher-trial-cache"
category = "trial.caches"
danger = "low"
after-cleanup = "skipped"
reason = "in-use"
"#;

    mod negative {
        use std::path::Path;

        use super::CASES;
        use crate::CaseFile;
        use crate::expected::checklist;

        #[test]
        fn a_negative_names_only_its_top_paths() {
            let text = checklist(&[CaseFile::parse(CASES).unwrap()], Path::new("/sandbox"));

            assert!(text.contains(
                "never offered: `home/work/fresher-trial-cache2`, `home/other.bin`; still there after the cleanup."
            ));
            assert!(!text.contains("look-alike.bin"));
        }
    }

    mod positive {
        use std::path::Path;

        use fresher_core::SkipCode;
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct Reason {
            reason: SkipCode,
        }

        // The names in the checklist are the names a case file's `reason` takes.
        #[test]
        fn every_skip_code_name_reads_back_as_its_code() {
            for code in [
                SkipCode::NotInSandbox,
                SkipCode::OutsideRoots,
                SkipCode::Protected,
                SkipCode::WholeFolder,
                SkipCode::Ignored,
                SkipCode::ThroughLink,
                SkipCode::Vanished,
                SkipCode::OtherVolume,
                SkipCode::Replaced,
                SkipCode::KindChanged,
                SkipCode::NotRemovable,
                SkipCode::MissingCapability,
                SkipCode::InUse,
                SkipCode::Grown,
                SkipCode::NoTrash,
                SkipCode::Covered,
                SkipCode::NotTheSubject,
                SkipCode::ForcedTool,
                SkipCode::CheckFailed,
            ] {
                let text = format!("reason = \"{}\"", crate::expected::code_name(code));
                assert_eq!(toml::from_str::<Reason>(&text).unwrap().reason, code);
            }
        }

        use super::CASES;
        use crate::CaseFile;
        use crate::expected::checklist;

        #[test]
        fn the_checklist_lists_every_case_negatives_first() {
            let text = checklist(&[CaseFile::parse(CASES).unwrap()], Path::new("/tmp/sandbox/trial"));

            let lines: Vec<&str> = text.lines().filter(|line| line.starts_with("- [ ] ")).collect();
            assert_eq!(
                lines,
                [
                    "- [ ] **a look-alike is never offered**: never offered: `home/work/fresher-trial-cache2`, \
                 `home/other.bin`; still there after the cleanup.",
                    "- [ ] **a cache goes to the Trash**: offered in *trial.caches* (Safe): \
                 `home/work/fresher-trial-cache`; in the sandbox's Trash after the cleanup.",
                    "- [ ] **a busy cache is skipped**: offered in *trial.caches* (Low): \
                 `home/busy/fresher-trial-cache`; skipped by the cleanup (`in-use`), still there.",
                ]
            );
            assert!(text.contains("From `fixtures/sandboxes/trial.toml`."));
            assert!(text.ends_with("```sh\nnpm run dev:sandbox -- /tmp/sandbox/trial\n```\n"));
        }
    }
}
