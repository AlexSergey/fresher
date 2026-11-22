use std::fmt::Write as _;
use std::path::Path;
use std::time::SystemTime;

use fresher_core::{Access, Capability, DangerLevel, Dictionary, ModuleId, SkipCode, Timestamp, core_dictionary};
use fresher_testing::{AppBundle, Stated, StatedFlag, StatedKind, StatedOwner};

use crate::generate::LeftOutFile;
use crate::world::folder_name;
use crate::{
    AfterCleanup, Case, CaseFile, CaseKind, Expectation, InsightExpectation, LargestExpectation, LeftOutExpectation,
    MapExpectation, MapPartName, Shown, Uninstall, UninstallExpectation,
};

/// `EXPECTED.md`: per case file, one checkbox per case, negatives first, each with one plain sentence and, under it,
/// its apps (installed, open, or found by Spotlight elsewhere), whether it turns Spotlight off, which guarded folders
/// it allows, refuses or has not been asked for, the facts its files state (which Finder never shows; times counted
/// from `now`), its left-outs, where the storage map shows it, what the Largest screen shows of it and what the Apps
/// screen shows of its app and its uninstall does; then the command that
/// opens the app on the sandbox at `sandbox`. A demo world lists only its scenery cases' names: nothing it shows is
/// checked. A case file left out of the sandbox is named after them, with the path it clashed on.
pub(crate) fn checklist(files: &[CaseFile], left_out: &[LeftOutFile], sandbox: &Path, now: SystemTime) -> String {
    let texts = core_dictionary().ok();
    let mut text = String::from(
        "# What the sandbox must show\n\nOpen the app on this sandbox, scan, clean everything it offers, and tick each case.\n",
    );
    for file in files {
        if file.demo().is_some() {
            let _ = write!(text, "\n## {}\n\nA demo world: what it shows is not checked.\n\n", file.feature());
            for case in file.cases() {
                let _ = writeln!(text, "- {}", case.name());
            }
            continue;
        }
        let _ = write!(text, "\n## {}\n\nFrom `fixtures/sandboxes/{}.toml`.\n\n", file.feature(), file.feature());
        let ordered = [CaseKind::Negative, CaseKind::Positive, CaseKind::Scenery]
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
            for &(capability, access) in case.access() {
                let _ = writeln!(text, "  - {}.", access_sentence(capability, access));
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
            if let Some(largest) = case.largest() {
                let _ = writeln!(text, "  - {}", largest_sentence(largest, texts.as_ref()));
            }
            if let Some(uninstall) = case.uninstall() {
                let _ = writeln!(text, "  - {}", uninstall_sentence(uninstall, case, texts.as_ref()));
            }
            for insight in case.insights() {
                let _ = writeln!(text, "  - {}", insight_sentence(insight));
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
        None if case.kind() == CaseKind::Positive => match case.uninstall() {
            Some(uninstall) => format!("uninstalled on the Apps screen: `{}`", uninstall.app().display()),
            None => String::new(),
        },
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

/// What the sandbox answers for a guarded folder or other apps' data, in words.
fn access_sentence(capability: Capability, access: Access) -> String {
    let folder = match capability {
        Capability::Folder(folder) => folder_name(folder),
        // A case states only the folders and other apps' data (`stated_capability`); the rest read the same.
        Capability::OtherAppsData
        | Capability::AllFiles
        | Capability::Volume(_)
        | Capability::AppManagement
        | Capability::Tool(_) => "other apps' data",
    };
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
    parts.extend(stated.last_used.map(|time| format!("last used {}", when(time, now))));
    parts.extend(
        stated.installed_by.as_ref().map(|by| format!("installed by {} as `{}`", by.manager().name(), by.token())),
    );
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

/// What the scan screen shows of an expected insight: its card, its numbers, its parts and their commands.
fn insight_sentence(insight: &InsightExpectation) -> String {
    let parts = match insight.parts() {
        0 => "its Details lists no part".to_owned(),
        parts => format!("its Details lists {parts} parts, {} with a command to copy", insight.commands()),
    };
    let copy = if insight.commands() > 0 { " Copy puts the command on the clipboard." } else { "" };
    format!(
        "Under «Described, not cleaned», the card «{}» shows {} numbers; {parts}.{copy} Nothing of it can be selected.",
        insight.identity(),
        insight.figures()
    )
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

/// What the Largest screen shows at the case's `at`, with the kind's name and the reason's sentence from the core's
/// dictionary (their ids when it cannot be read).
fn largest_sentence(largest: &LargestExpectation, texts: Option<&Dictionary>) -> String {
    let text = |key: fresher_core::TextKey| {
        texts.and_then(|texts| texts.get(key.as_str())).map_or_else(|| key.as_str().to_owned(), str::to_owned)
    };
    let at = largest.at().display();
    match largest.shown() {
        Shown::Offered => {
            let kind = largest.kind().map(|kind| format!(" as *{}*", text(kind.text_key()))).unwrap_or_default();
            let unpacked =
                largest.unpacked().map(|folder| format!(" It names `{folder}` beside it.")).unwrap_or_default();
            format!("On Largest, `{at}` is listed{kind} and can be chosen.{unpacked}")
        }
        Shown::Absent => format!("On Largest, nothing at or below `{at}` is listed."),
        Shown::Withheld => {
            let reason = largest.withheld().map(|reason| text(reason.text_key())).unwrap_or_default();
            format!("On Largest → Folders, `{at}` cannot be chosen: {}.", reason.trim_end_matches('.'))
        }
        Shown::NotSeen => format!(
            "On Largest, nothing in `{at}` is listed, and the notice above the list says Fresher did not look there \
             and how to allow it."
        ),
    }
}

/// What the Apps screen shows of the case's app and what its uninstall does, with the reason's sentence and the
/// choice's label from the core's dictionary (their ids when it cannot be read); the app is named as its bundle names
/// it.
fn uninstall_sentence(uninstall: &UninstallExpectation, case: &Case, texts: Option<&Dictionary>) -> String {
    let text = |key: fresher_core::TextKey| {
        texts.and_then(|texts| texts.get(key.as_str())).map_or_else(|| key.as_str().to_owned(), str::to_owned)
    };
    let app = case.tree().apps().iter().find(|app| app.path == uninstall.app());
    let name = app.map_or_else(|| uninstall.app().display().to_string(), |app| app.name.clone());
    let paths = |paths: &[std::path::PathBuf]| -> String {
        paths.iter().map(|path| format!("`{}`", path.display())).collect::<Vec<_>>().join(", ")
    };
    match uninstall.expected() {
        Uninstall::NotListed => format!("Apps: `{name}` is not listed."),
        Uninstall::NotOffered(reason) => {
            let manager = app.and_then(|app| app.manager().ok().flatten());
            let manager = manager.map_or(fresher_core::Manager::Homebrew.name(), |by| by.manager().name());
            let sentence = text(reason.text_key()).replace("{manager}", manager);
            format!("Apps: `{name}` is shown and not offered: {sentence}")
        }
        Uninstall::Chosen { choice, trashed, kept } => {
            let stay = if kept.is_empty() { "nothing".to_owned() } else { paths(kept) };
            format!(
                "Apps: uninstall `{name}` with \"{}\": these go to the Trash: {}; these stay: {stay}. Then Cleanups → \
                 put back: all of them are back.",
                text(choice.label_key()),
                paths(trashed)
            )
        }
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
        fn a_case_without_a_largest_table_says_nothing_of_largest() {
            let text = checklist(&[CaseFile::parse(CASES).unwrap()], &[], Path::new("/sandbox"), super::now());

            assert!(!text.contains("Largest"), "{text}");
        }

        #[test]
        fn without_the_dictionary_the_kind_and_the_reason_are_named_by_their_keys() {
            let cases = "feature = \"large-files\"\n[[case]]\nname = \"a\"\nkind = \"negative\"\n[[case.file]]\n\
                         path = \"home/Movies/b.txt\"\n[case.largest]\nshown = \"withheld\"\nat = \"home/Movies\"\n\
                         withheld = \"standard-folder\"\n";
            let parsed = CaseFile::parse(cases).unwrap();

            let sentence = crate::expected::largest_sentence(parsed.cases()[0].largest().unwrap(), None);

            assert_eq!(
                sentence,
                "On Largest → Folders, `home/Movies` cannot be chosen: core.largest.withheld.standard-folder."
            );
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
        fn the_checklist_says_what_largest_shows() {
            let cases = format!(
                "feature = \"large-files\"\n{}{}{}{}",
                "[[case]]\nname = \"a\"\nkind = \"negative\"\n[[case.file]]\npath = \"home/Library/a.bin\"\n\
                 [case.largest]\nshown = \"absent\"\n",
                "[[case]]\nname = \"b\"\nkind = \"negative\"\n[[case.file]]\npath = \"home/Movies/b.txt\"\n\
                 [case.largest]\nshown = \"withheld\"\nat = \"home/Movies\"\nwithheld = \"standard-folder\"\n",
                "[[case]]\nname = \"c\"\nkind = \"negative\"\n[[case.file]]\npath = \"home/Documents/c.iso\"\n\
                 [case.largest]\nshown = \"not-seen\"\nat = \"home/Documents\"\n",
                "[[case]]\nname = \"d\"\nkind = \"positive\"\n[[case.file]]\npath = \"home/A/Photos.zip\"\n\
                 [case.expect]\nitem = \"home/A/Photos.zip\"\ncategory = \"core.own-file\"\ndanger = \"high\"\n\
                 after-cleanup = \"in-trash\"\n[case.largest]\nshown = \"offered\"\nkind = \"archive\"\n\
                 unpacked = \"Photos\"\n[[case]]\nname = \"e\"\nkind = \"positive\"\n[[case.file]]\n\
                 path = \"home/B/x.bin\"\n[case.expect]\nitem = \"home/B\"\ncategory = \"core.own-file\"\n\
                 danger = \"high\"\nafter-cleanup = \"in-trash\"\n[case.largest]\nshown = \"offered\"\nat = \"home/B\"\n",
            );

            let text = checklist(&[CaseFile::parse(&cases).unwrap()], &[], Path::new("/sandbox"), super::now());

            let lines: Vec<&str> = text.lines().filter(|line| line.starts_with("  - ")).collect();
            assert_eq!(
                lines,
                [
                    "  - On Largest, nothing at or below `home/Library/a.bin` is listed.",
                    "  - On Largest → Folders, `home/Movies` cannot be chosen: A standard folder of your Mac. Open it to \
                     choose what is inside.",
                    "  - On Largest, nothing in `home/Documents` is listed, and the notice above the list says Fresher \
                     did not look there and how to allow it.",
                    "  - On Largest, `home/A/Photos.zip` is listed as *Archive* and can be chosen. It names `Photos` \
                     beside it.",
                    "  - On Largest, `home/B` is listed and can be chosen.",
                ]
            );
        }

        #[test]
        fn the_checklist_says_what_the_apps_screen_shows_and_the_uninstall_does() {
            let app = |path: &str, id: &str, name: &str, extra: &str| {
                format!("[[case.app]]\npath = \"{path}\"\nbundle-id = \"{id}\"\nname = \"{name}\"\n{extra}")
            };
            let brewed =
                app("Applications/Brewed.app", "com.example.Brewed", "Brewed", "installed-by = \"homebrew:brewed\"\n");
            let keeper = app("Applications/Keeper.app", "com.example.Keeper", "Keeper", "last-used = \"240d\"\n");
            let cases = [
                "feature = \"uninstall\"\nmodule = \"trial\"\n".to_owned(),
                format!(
                    "[[case]]\nname = \"a\"\nkind = \"negative\"\n{}[case.uninstall]\n\
                     app = \"Applications/Notes.app\"\nlisted = false\n",
                    app("Applications/Notes.app", "com.apple.Notes", "Notes", "")
                ),
                format!(
                    "[[case]]\nname = \"b\"\nkind = \"negative\"\n{brewed}[case.uninstall]\n\
                     app = \"Applications/Brewed.app\"\noffered = false\nreason = \"installed-by\"\n"
                ),
                format!(
                    "[[case]]\nname = \"c\"\nkind = \"positive\"\n{keeper}[[case.file]]\n\
                     path = \"home/Library/Caches/com.example.Keeper/a.bin\"\n[[case.file]]\n\
                     path = \"home/Library/Preferences/com.example.Keeper.plist\"\n[case.uninstall]\n\
                     app = \"Applications/Keeper.app\"\nchoice = \"keep-data\"\n\
                     trashed = [\"Applications/Keeper.app\", \"home/Library/Caches/com.example.Keeper\"]\n\
                     kept = [\"home/Library/Preferences/com.example.Keeper.plist\"]\n"
                ),
                format!(
                    "[[case]]\nname = \"d\"\nkind = \"positive\"\n{}[case.uninstall]\n\
                     app = \"Applications/Purger.app\"\nchoice = \"everything\"\ntrashed = [\"Applications/Purger.app\"]\n",
                    app("Applications/Purger.app", "com.example.Purger", "Purger", "")
                ),
            ]
            .concat();

            let text = checklist(&[CaseFile::parse(&cases).unwrap()], &[], Path::new("/sandbox"), super::now());

            let lines: Vec<&str> = text.lines().filter(|line| line.starts_with("  - Apps")).collect();
            assert_eq!(
                lines,
                [
                    "  - Apps: `Notes` is not listed.",
                    "  - Apps: `Brewed` is shown and not offered: Homebrew installed this app. Uninstall it with \
                     Homebrew, so it stays in step.",
                    "  - Apps: uninstall `Keeper` with \"Remove the app, keep its settings and data\": these go to the \
                     Trash: `Applications/Keeper.app`, `home/Library/Caches/com.example.Keeper`; these stay: \
                     `home/Library/Preferences/com.example.Keeper.plist`. Then Cleanups → put back: all of them are back.",
                    "  - Apps: uninstall `Purger` with \"Remove the app and everything it left\": these go to the Trash: \
                     `Applications/Purger.app`; these stay: nothing. Then Cleanups → put back: all of them are back.",
                ]
            );
            assert!(text.contains("**c**: uninstalled on the Apps screen: `Applications/Keeper.app`."), "{text}");
            assert!(
                text.contains("  - `Applications/Brewed.app` stated: installed by Homebrew as `brewed`."),
                "{text}"
            );
            assert!(text.contains("  - `Applications/Keeper.app` stated: last used 240 days ago."), "{text}");
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
        fn an_expected_insight_writes_its_line() {
            let insights = "[[case.insight]]\nidentity = \"orbstack\"\nfigures = 3\nparts = 4\ncommands = 3\n\
                            [[case.insight]]\nidentity = \"docker-desktop\"\nfigures = 2\nparts = 0\ncommands = 0\n";
            let cases = CASES.replace("path = \"home/other.bin\"\n", &format!("path = \"home/other.bin\"\n{insights}"));

            let text = checklist(&[CaseFile::parse(&cases).unwrap()], &[], Path::new("/sandbox"), super::now());

            let lines: Vec<&str> = text.lines().filter(|line| line.starts_with("  - ")).collect();
            assert_eq!(
                lines,
                [
                    "  - Under «Described, not cleaned», the card «orbstack» shows 3 numbers; its Details lists 4 parts, \
                     3 with a command to copy. Copy puts the command on the clipboard. Nothing of it can be selected.",
                    "  - Under «Described, not cleaned», the card «docker-desktop» shows 2 numbers; its Details lists no \
                     part. Nothing of it can be selected.",
                ]
            );
        }

        #[test]
        fn the_checklist_names_other_apps_data() {
            let access = "access = { other-apps-data = \"denied\" }\n";
            let cases = CASES.replacen("kind = \"negative\"\n", &format!("kind = \"negative\"\n{access}"), 1);

            let text = checklist(&[CaseFile::parse(&cases).unwrap()], &[], Path::new("/sandbox"), super::now());

            assert!(text.lines().any(|line| line == "  - The sandbox refuses other apps' data."), "{text}");
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

        #[test]
        fn a_demo_world_lists_its_scenery_cases_and_checks_nothing() {
            let world = "feature = \"developer-mac\"\n[demo]\nnow = 1795000000\nshows = [\"trial.caches\"]\n\
                         [[case]]\nname = \"Xcode's build products\"\nkind = \"scenery\"\n\
                         [[case.file]]\npath = \"home/Library/Developer/Atlas.bin\"\nsize = 4096\n\
                         [[case]]\nname = \"a browser's cache\"\nkind = \"scenery\"\n\
                         [[case.file]]\npath = \"home/Library/Caches/cache.bin\"\nage = \"3d\"\n";

            let text = checklist(
                &[CaseFile::parse(world).unwrap()],
                &[],
                Path::new("/private/tmp/fresher-demo/overview"),
                super::now(),
            );

            assert_eq!(
                text,
                "# What the sandbox must show\n\nOpen the app on this sandbox, scan, clean everything it offers, and \
                 tick each case.\n\n## developer-mac\n\nA demo world: what it shows is not checked.\n\n\
                 - Xcode's build products\n- a browser's cache\n\n## Open the app on it\n\n```sh\n\
                 npm run dev:sandbox -- /private/tmp/fresher-demo/overview\n```\n"
            );
        }
    }
}
