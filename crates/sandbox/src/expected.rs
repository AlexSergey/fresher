use std::fmt::Write as _;
use std::path::Path;
use std::time::SystemTime;

use fresher_core::{Access, DangerLevel, GuardedFolder, ModuleId, SkipCode, Timestamp};
use fresher_testing::{AppBundle, Stated, StatedFlag, StatedKind, StatedOwner};

use crate::generate::LeftOutFile;
use crate::world::folder_name;
use crate::{AfterCleanup, Case, CaseFile, CaseKind, Expectation, LeftOutExpectation, MapExpectation, MapPartName};

/// `EXPECTED.md`: per case file, one checkbox per case, negatives first, each with one plain sentence and, under it,
/// its apps (installed, open, or found by Spotlight elsewhere), whether it turns Spotlight off, which guarded folders
/// it allows, refuses or has not been asked for, the facts its files state (which Finder never shows; times counted
/// from `now`), its left-outs and where the storage map shows it; then the command that opens the app on the sandbox
/// at `sandbox`. A case file left out of the sandbox is named after them, with the path it clashed on.
pub(crate) fn checklist(files: &[CaseFile], left_out: &[LeftOutFile], sandbox: &Path, now: SystemTime) -> String {
    let mut text = String::from(
        "# What the sandbox must show\n\nOpen the app on this sandbox, scan, clean everything it offers, and tick each case.\n",
    );
    for file in files {
        let _ = write!(text, "\n## {}\n\nFrom `fixtures/sandboxes/{}.toml`.\n\n", file.feature(), file.feature());
        let ordered = [CaseKind::Negative, CaseKind::Positive]
            .into_iter()
            .flat_map(|kind| file.cases().iter().filter(move |case| case.kind() == kind));
        for case in ordered {
            let _ = writeln!(text, "- [ ] **{}**: {}.", case.name(), sentence(case, file.module()));
            for app in case.tree().apps() {
                let open = case.running_apps().any(|running| running.path == app.path);
                let _ = writeln!(text, "  - {}{}.", installed(app), if open { " (open)" } else { "" });
            }
            for app in case.running_elsewhere() {
                let _ = writeln!(text, "  - Open, not installed: {}.", app.name);
            }
            for (name, _) in case.elsewhere() {
                let _ = writeln!(text, "  - Found by Spotlight outside the apps folders: {name}.");
            }
            if case.spotlight_off() {
                let _ = writeln!(
                    text,
                    "  - Spotlight is off: it cannot be asked in this sandbox, for every case file in it."
                );
            }
            for &(folder, access) in case.access() {
                let _ = writeln!(text, "  - {}.", access_sentence(folder, access));
            }
            if let Some(version) = case.macos() {
                let _ = writeln!(text, "  - The sandbox runs macOS {}.{}.", version.major, version.minor);
            }
            for (package, age) in case.receipts() {
                let _ =
                    writeln!(text, "  - Installer receipt: {package}, installed {age} before the sandbox was made.");
            }
            for (path, stated) in case.tree().stated(now).unwrap_or_default() {
                let _ = writeln!(text, "  - `{}` stated: {}.", path.display(), stated_sentence(&stated, now));
            }
            for left_out in case.left_out() {
                let _ = writeln!(text, "  - {}.", left_out_sentence(left_out));
            }
            if let Some(map) = case.map() {
                let _ = writeln!(text, "  - {}.", map_sentence(map));
            }
        }
    }
    if !left_out.is_empty() {
        text.push_str("\n## Case files left out\n\n");
        for file in left_out {
            let LeftOutFile { feature, first, path } = file;
            let _ = writeln!(
                text,
                "- `{feature}` is not in this sandbox: it writes `{}`, which `{first}` writes too; write it by name: \
                 `npm run sandbox -- {feature}`.",
                path.display()
            );
        }
    }
    let _ = write!(text, "\n## Open the app on it\n\n```sh\nnpm run dev:sandbox -- {}\n```\n", sandbox.display());
    text
}

/// What must happen to the case, in one sentence. A negative of a file that names its module reads "never offered by
/// `<module>`", one of a file with no module "never offered".
fn sentence(case: &Case, module: Option<ModuleId>) -> String {
    match case.expect() {
        None => {
            let by = module.map_or_else(String::new, |module| format!(" by `{}`", module.as_str()));
            format!("never offered{by}: {}; still there after the cleanup", top_paths(case))
        }
        Some(expect) => format!(
            "offered in *{}* ({}): `{}`; {}",
            expect.category().as_str(),
            danger_name(expect.danger()),
            expect.item().display(),
            after(expect)
        ),
    }
}

/// An app the case installs: `Installed: <name> <version>`, followed by ` (open)` when it runs.
fn installed(app: &AppBundle) -> String {
    match &app.version {
        Some(version) => format!("Installed: {} {version}", app.name),
        None => format!("Installed: {}", app.name),
    }
}

/// What the case's left-out expectation says: the category and the reason are named by their ids, which the sandbox
/// crate has no dictionary to turn into the module's texts.
fn left_out_sentence(left_out: &LeftOutExpectation) -> String {
    format!("Under '{}', {} left out: {}", left_out.category().as_str(), left_out.items(), left_out.reason().as_str())
}

/// What the sandbox answers for a guarded folder, in words.
fn access_sentence(folder: GuardedFolder, access: Access) -> String {
    let folder = folder_name(folder);
    match access {
        // A case states only the first three; the other two never reach a checklist and read as allowed and refused.
        Access::Granted | Access::NotApplicable => {
            format!("The sandbox allows {folder}; it is read once its switch in Settings → Permissions is on")
        }
        Access::Denied | Access::Unknown => format!("The sandbox refuses {folder}"),
        Access::NotAsked => format!(
            "The sandbox has not been asked for {folder}: turn its switch on in Settings → Permissions first, and \
             the sandbox allows it"
        ),
    }
}

/// What a path states instead of having it, in words: its kind, owner, flags and times.
fn stated_sentence(stated: &Stated, now: SystemTime) -> String {
    let mut parts = Vec::new();
    parts.extend(stated.kind.map(|kind| match kind {
        StatedKind::Socket => "a socket".to_owned(),
        StatedKind::Pipe => "a named pipe".to_owned(),
    }));
    parts.extend(stated.owner.map(|owner| match owner {
        StatedOwner::System => "owned by the system".to_owned(),
        StatedOwner::OtherUser => "owned by another user".to_owned(),
    }));
    parts.extend(stated.flags.iter().map(|flag| {
        match flag {
            StatedFlag::UserLock => "locked",
            StatedFlag::SystemLock => "locked by the system",
            StatedFlag::NoUnlink => "not removable",
            StatedFlag::Restricted => "guarded by the system",
        }
        .to_owned()
    }));
    parts.extend(stated.changed.map(|time| format!("changed {}", when(time, now))));
    parts.extend(stated.accessed.map(|time| format!("last read {}", when(time, now))));
    parts.join(", ")
}

/// `time` from `now`: "30 days ago", "2 hours ago", "90 seconds ago", or "in 2 days" after it.
fn when(time: Timestamp, now: SystemTime) -> String {
    let now = Timestamp::from_system_time(now).secs();
    let (seconds, after) = if time.secs() > now { (time.secs() - now, true) } else { (now - time.secs(), false) };
    let (count, unit) = match seconds {
        0 => return "now".into(),
        _ if seconds % 86_400 == 0 => (seconds / 86_400, "day"),
        _ if seconds % 3_600 == 0 => (seconds / 3_600, "hour"),
        _ => (seconds, "second"),
    };
    let unit = if count == 1 { unit.to_owned() } else { format!("{unit}s") };
    if after { format!("in {count} {unit}") } else { format!("{count} {unit} ago") }
}

/// Where the overview's by-place view shows the case's files.
fn map_sentence(map: &MapExpectation) -> String {
    let at = map.at().display();
    match map.part() {
        MapPartName::YourFolders => {
            format!("on the overview, by place: `{at}` is in *Your folders*, nothing of it explained")
        }
        MapPartName::Explained => format!("on the overview, by place: `{at}` is explained"),
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
        SkipCode::DryRun => "dry-run",
        SkipCode::CheckFailed => "check-failed",
    }
}

#[cfg(test)]
mod tests {
    fn now() -> std::time::SystemTime {
        std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(fresher_testing::FIXTURE_NOW)
    }

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
            let text = checklist(&[CaseFile::parse(CASES).unwrap()], &[], Path::new("/sandbox"), super::now());

            assert!(text.contains(
                "never offered: `home/work/fresher-trial-cache2`, `home/other.bin`; still there after the cleanup."
            ));
            assert!(!text.contains("look-alike.bin"));
        }

        #[test]
        fn a_negative_of_a_file_with_a_module_names_the_module() {
            let cases = CASES.replace("feature = \"trial\"\n", "feature = \"trial\"\nmodule = \"trial\"\n");

            let text = checklist(&[CaseFile::parse(&cases).unwrap()], &[], Path::new("/sandbox"), super::now());

            assert!(text.contains(
                "never offered by `trial`: `home/work/fresher-trial-cache2`, `home/other.bin`; still there after the cleanup."
            ));
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
                SkipCode::DryRun,
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
        fn a_stated_fact_is_named_under_its_case() {
            let cases = CASES.replace(
                "path = \"home/other.bin\"\n",
                "path = \"home/other.bin\"\nowner = \"system\"\nchanged = \"30d\"\n\
                 [[case.file]]\npath = \"private/tmp/sock\"\nkind = \"socket\"\nflags = [\"no-unlink\", \"user-lock\"]\n\
                 accessed = \"-2h\"\n[[case.dir]]\npath = \"private/tmp/theirs\"\nowner = \"other-user\"\nchanged = \"90s\"\n",
            );

            let text = checklist(&[CaseFile::parse(&cases).unwrap()], &[], Path::new("/sandbox"), super::now());

            for line in [
                "  - `private/tmp/theirs` stated: owned by another user, changed 90 seconds ago.",
                "  - `home/other.bin` stated: owned by the system, changed 30 days ago.",
                "  - `private/tmp/sock` stated: a socket, locked, not removable, last read in 2 hours.",
            ] {
                assert!(text.lines().any(|found| found == line), "{line}\n{text}");
            }
        }

        #[test]
        fn the_checklist_says_where_the_map_shows_it() {
            let cases = CASES
                .replace(
                    "path = \"home/other.bin\"\n",
                    "path = \"home/other.bin\"\n[case.map]\npart = \"your-folders\"\nat = \"home/work\"\n",
                )
                .replace(
                    "after-cleanup = \"in-trash\"\n",
                    "after-cleanup = \"in-trash\"\n[case.map]\npart = \"explained\"\n",
                );

            let text = checklist(&[CaseFile::parse(&cases).unwrap()], &[], Path::new("/sandbox"), super::now());

            let lines: Vec<&str> =
                text.lines().filter(|line| line.starts_with("- [ ] ") || line.starts_with("  - ")).collect();
            assert_eq!(lines.len(), 5, "{text}");
            assert!(lines[0].starts_with("- [ ] **a look-alike is never offered**"));
            assert_eq!(
                lines[1],
                "  - on the overview, by place: `home/work` is in *Your folders*, nothing of it explained."
            );
            assert!(lines[2].starts_with("- [ ] **a cache goes to the Trash**"));
            assert_eq!(lines[3], "  - on the overview, by place: `home/work/fresher-trial-cache` is explained.");
            assert!(lines[4].starts_with("- [ ] **a busy cache is skipped**"));
        }

        #[test]
        fn the_checklist_lists_a_cases_apps() {
            let cases = CASES.replace(
                "path = \"home/other.bin\"\n",
                "path = \"home/other.bin\"\n[[case.app]]\npath = \"Applications/WebStorm.app\"\n\
                 bundle-id = \"com.jetbrains.WebStorm\"\nname = \"WebStorm\"\nversion = \"2025.3.1\"\n\
                 running = true\n[[case.app]]\npath = \"Applications/Plain.app\"\nbundle-id = \"com.example.Plain\"\n\
                 name = \"Plain\"\n[[case.running]]\nname = \"CLion\"\nbundle-ids = [\"com.jetbrains.CLion\"]\n",
            );

            let text = checklist(&[CaseFile::parse(&cases).unwrap()], &[], Path::new("/sandbox"), super::now());

            let lines: Vec<&str> = text.lines().filter(|line| line.starts_with("  - ")).collect();
            assert_eq!(
                lines,
                [
                    "  - Installed: WebStorm 2025.3.1 (open).",
                    "  - Installed: Plain.",
                    "  - Open, not installed: CLion."
                ]
            );
        }

        #[test]
        fn the_checklist_names_the_apps_spotlight_finds_elsewhere_and_spotlight_off() {
            let cases = CASES
                .replace(
                    "path = \"home/other.bin\"\n",
                    "path = \"home/other.bin\"\n[[case.elsewhere]]\nname = \"Far\"\nbundle-id = \"com.example.Far\"\n",
                )
                .replace("kind = \"negative\"\n", "kind = \"negative\"\nspotlight = \"off\"\n");

            let text = checklist(&[CaseFile::parse(&cases).unwrap()], &[], Path::new("/sandbox"), super::now());

            let lines: Vec<&str> = text.lines().filter(|line| line.starts_with("  - ")).collect();
            assert_eq!(
                lines,
                [
                    "  - Found by Spotlight outside the apps folders: Far.",
                    "  - Spotlight is off: it cannot be asked in this sandbox, for every case file in it."
                ]
            );
        }

        #[test]
        fn the_checklist_says_what_the_guarded_folders_allow() {
            let access = "access = { downloads = \"granted\", desktop = \"denied\", documents = \"not-asked\" }\n";
            let cases = CASES.replacen("kind = \"negative\"\n", &format!("kind = \"negative\"\n{access}"), 1);

            let text = checklist(&[CaseFile::parse(&cases).unwrap()], &[], Path::new("/sandbox"), super::now());

            let lines: Vec<&str> = text.lines().filter(|line| line.starts_with("  - ")).collect();
            assert_eq!(
                lines,
                [
                    "  - The sandbox refuses Desktop.",
                    "  - The sandbox has not been asked for Documents: turn its switch on in Settings → Permissions \
                     first, and the sandbox allows it.",
                    "  - The sandbox allows Downloads; it is read once its switch in Settings → Permissions is on.",
                ]
            );
        }

        #[test]
        fn the_checklist_names_the_macos_version_and_the_receipts() {
            let installs = "macos = \"15.4\"\n";
            let cases = CASES.replacen("kind = \"negative\"\n", &format!("kind = \"negative\"\n{installs}"), 1);
            let cases = format!("{cases}[[case.receipt]]\npackage = \"Tool-3.1.pkg\"\ninstalled = \"10d\"\n");

            let text = checklist(&[CaseFile::parse(&cases).unwrap()], &[], Path::new("/sandbox"), super::now());

            let lines: Vec<&str> = text.lines().filter(|line| line.starts_with("  - ")).collect();
            assert!(lines.contains(&"  - The sandbox runs macOS 15.4."), "{lines:?}");
            assert!(
                lines.contains(&"  - Installer receipt: Tool-3.1.pkg, installed 10d before the sandbox was made."),
                "{lines:?}"
            );
        }

        #[test]
        fn the_checklist_says_what_is_left_out() {
            let cases = CASES.replace(
                "path = \"home/other.bin\"\n",
                "path = \"home/other.bin\"\n[[case.left-out]]\ncategory = \"trial.caches\"\n\
                 reason = \"trial.error.failed\"\nitems = 2\n",
            );

            let text = checklist(&[CaseFile::parse(&cases).unwrap()], &[], Path::new("/sandbox"), super::now());

            let lines: Vec<&str> = text.lines().filter(|line| line.starts_with("  - ")).collect();
            assert_eq!(lines, ["  - Under 'trial.caches', 2 left out: trial.error.failed."]);
        }

        #[test]
        fn the_checklist_lists_every_case_negatives_first() {
            let text =
                checklist(&[CaseFile::parse(CASES).unwrap()], &[], Path::new("/tmp/sandbox/trial"), super::now());

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
