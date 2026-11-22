//! `fresher-sandbox generate <feature|all|path.toml> [--into <dir>]`: writes the case files of a feature
//! (`fixtures/sandboxes/<feature>.toml`, every file there for `all`), or one case file read by its path (a demo world
//! of `fixtures/demos/` included), into a sandbox folder, `sandbox/<feature or file stem>/` at the repository's root by
//! default. A demo world is generated at its own clock. Dev only: `npm run sandbox -- <feature>`.
//!
//! `fresher-sandbox demo <scenario> [--into <dir>]`: writes the world of the demo scenario
//! `fixtures/demos/scenarios/<scenario>.toml` into a new folder, `/private/tmp/fresher-demo/<scenario>-<n>` by default
//! (the first `n` not taken), and prints that folder; it never removes or overwrites anything.

#![forbid(unsafe_code)]

use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;
use std::time::SystemTime;

use fresher_core::{Protection, ProtectionError};
use fresher_sandbox::{CaseFile, Demo, generate_leaving_out, without_clashes};

mod demo_command;

const USAGE: &str = "usage: fresher-sandbox generate <feature|all|path.toml> [--into <dir>]\n       \
                     fresher-sandbox demo <scenario> [--into <dir>]";
/// The identifiers of the release app and of the development build: both data folders are protected like the app's
/// own.
const APP_IDENTIFIERS: [&str; 2] = ["io.github.alexsergey.fresher", "io.github.alexsergey.fresher.dev"];

/// What to generate, read from the arguments.
#[derive(Debug, PartialEq, Eq)]
enum Command {
    /// `generate`: a feature name, `all`, or the path of a case file.
    Generate { target: String, into: Option<PathBuf> },
    /// `demo`: the name of a demo scenario.
    Demo { scenario: String, into: Option<PathBuf> },
}

fn main() -> ExitCode {
    match run(std::env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            report_error(&message);
            ExitCode::FAILURE
        }
    }
}

fn run(args: impl Iterator<Item = OsString>) -> Result<(), String> {
    let command = parse_args(args)?;
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let repository = fs::canonicalize(&repository).map_err(|error| format!("{}: {error}", repository.display()))?;
    // The binary's one home lookup: only for the real Mac's protection list, which keeps the sandbox off the home.
    #[expect(clippy::disallowed_methods, reason = "the home folder is looked up once per binary, here")]
    let home = std::env::home_dir().ok_or("the home folder is unknown")?;
    let protection = real_protection(&home).map_err(|error| error.to_string())?;
    match command {
        Command::Generate { target, into } => run_generate(&repository, &target, into, &protection),
        Command::Demo { scenario, into } => {
            let demos = repository.join("fixtures").join("demos");
            let written = demo_command::demo(
                &demos,
                &scenario,
                into,
                Path::new(demo_command::DEMO_FOLDER),
                &protection,
                fresher_macos::current_user(),
            )?;
            report_demo(&written.world, &written.folder);
            Ok(())
        }
    }
}

/// `generate`: the case files of `target` into `into`, by default `sandbox/<feature or file stem>/`.
fn run_generate(repository: &Path, target: &str, into: Option<PathBuf>, protection: &Protection) -> Result<(), String> {
    let files = case_files(&repository.join("fixtures").join("sandboxes"), target)?;
    let (files, left_out) =
        if target == "all" { without_clashes(files).map_err(|error| error.to_string())? } else { (files, Vec::new()) };
    for file in &left_out {
        report_left_out(&file.feature, &file.first, &file.path);
    }
    let into = into.unwrap_or_else(|| repository.join("sandbox").join(folder_name(target)));
    // A demo world has its own clock; any other case file is generated now.
    let now = files.iter().find_map(CaseFile::demo).map_or_else(SystemTime::now, Demo::now);
    let generated = generate_leaving_out(&files, &left_out, &into, now, protection, fresher_macos::current_user())
        .map_err(|error| error.to_string())?;
    report(&files, generated.sandbox.path(), &generated.strangers);
    Ok(())
}

/// The real Mac's protection list for `home`, with the data folders of both the release app and the development
/// build, and the user's own temporary folder when the system names it.
fn real_protection(home: &Path) -> Result<Protection, ProtectionError> {
    let user_temp = fresher_macos::user_temp_dir().ok();
    let [release, development] = APP_IDENTIFIERS.map(|identifier| {
        let data_folder = home.join("Library").join("Application Support").join(identifier);
        fresher_macos::protection(home, &data_folder, None, user_temp.as_deref())
    });
    release?.joined(&development?)
}

/// Reads `generate <feature|all|path.toml> [--into <dir>]` or `demo <scenario> [--into <dir>]`; a path is kept as
/// given.
fn parse_args(mut args: impl Iterator<Item = OsString>) -> Result<Command, String> {
    let verb = args.next();
    let name = args.next().and_then(|name| name.into_string().ok()).ok_or(USAGE)?;
    let into = match (args.next(), args.next(), args.next()) {
        (None, ..) => None,
        (Some(flag), Some(dir), None) if flag == "--into" => Some(PathBuf::from(dir)),
        _ => return Err(USAGE.into()),
    };
    match verb.as_deref().and_then(OsStr::to_str) {
        Some("generate") => Ok(Command::Generate { target: name, into }),
        Some("demo") => Ok(Command::Demo { scenario: name, into }),
        _ => Err(USAGE.into()),
    }
}

/// Whether the target names a case file by its path: it holds a `/` or ends in `.toml`.
fn is_path(target: &str) -> bool {
    target.contains('/') || Path::new(target).extension().is_some_and(|extension| extension == "toml")
}

/// The default sandbox folder's name: the file stem of a path, else the feature (or `all`).
fn folder_name(target: &str) -> &OsStr {
    if is_path(target) { Path::new(target).file_stem().unwrap_or(OsStr::new(target)) } else { OsStr::new(target) }
}

/// The case files of `target`: the one of a feature in `fixtures`, every one there by name for `all` (never the demo
/// worlds of `fixtures/demos/`), or the one at a path, relative to the current folder.
fn case_files(fixtures: &Path, target: &str) -> Result<Vec<CaseFile>, String> {
    let paths = if is_path(target) {
        vec![PathBuf::from(target)]
    } else if target == "all" {
        let entries = fs::read_dir(fixtures).map_err(|error| format!("{}: {error}", fixtures.display()))?;
        let mut paths: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "toml"))
            .collect();
        // By file stem, so a base file precedes its `-variant` files, whose clashes `all` then leaves out.
        paths.sort_by(|a, b| a.file_stem().cmp(&b.file_stem()));
        paths
    } else {
        let mut name = Path::new(target).components();
        if !matches!((name.next(), name.next()), (Some(Component::Normal(_)), None)) {
            return Err(format!("`{target}` is not a feature name"));
        }
        vec![fixtures.join(format!("{target}.toml"))]
    };
    if paths.is_empty() {
        return Err(format!("no case file in {}", fixtures.display()));
    }
    paths
        .iter()
        .map(|path| {
            let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
            CaseFile::parse(&text).map_err(|error| format!("{}: {error}", path.display()))
        })
        .collect()
}

#[expect(clippy::print_stdout, reason = "the command's output")]
fn report(files: &[CaseFile], sandbox: &Path, strangers: &[PathBuf]) {
    let cases: usize = files.iter().map(|file| file.cases().len()).sum();
    println!("wrote {cases} cases into {} (its checklist: EXPECTED.md)", sandbox.display());
    if !strangers.is_empty() {
        println!("not made by the generator, left as they are:");
        for stranger in strangers {
            println!("  {}", stranger.display());
        }
    }
}

#[expect(clippy::print_stdout, reason = "the command's output")]
fn report_demo(world: &str, folder: &Path) {
    println!("wrote the demo world {world} into {}", folder.display());
}

#[expect(clippy::print_stderr, reason = "the command's warning")]
fn report_left_out(feature: &str, first: &str, path: &Path) {
    eprintln!(
        "fresher-sandbox: `{feature}` is left out of `all`: it writes {}, which `{first}` writes too; \
         write it by name: npm run sandbox -- {feature}",
        path.display()
    );
}

#[expect(clippy::print_stderr, reason = "the command's error")]
fn report_error(message: &str) {
    eprintln!("fresher-sandbox: {message}");
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    fn args(words: &[&str]) -> impl Iterator<Item = OsString> {
        words.iter().map(OsString::from).collect::<Vec<_>>().into_iter()
    }

    const CASE_FILE: &str = "feature = \"trial\"\n[[case]]\nname = \"n\"\nkind = \"negative\"\n";

    mod negative {
        use std::fs;

        use super::args;
        use crate::{APP_IDENTIFIERS, USAGE, case_files, parse_args, real_protection};

        #[test]
        fn wrong_arguments_print_the_usage() {
            for words in [
                &[][..],
                &["build", "trial"],
                &["generate"],
                &["generate", "trial", "--into"],
                &["generate", "trial", "--to", "x"],
                &["generate", "trial", "--into", "x", "y"],
                &["demo"],
                &["demo", "overview", "--into"],
                &["demo", "overview", "--to", "x"],
            ] {
                assert_eq!(parse_args(args(words)), Err(USAGE.into()), "{words:?}");
            }
        }

        #[test]
        fn both_data_folders_of_the_app_are_protected() {
            let folder = tempfile::tempdir().unwrap();
            let home = fs::canonicalize(folder.path()).unwrap();
            let protection = real_protection(&home).unwrap();

            for identifier in APP_IDENTIFIERS {
                let data_folder = home.join("Library").join("Application Support").join(identifier);
                assert!(protection.check(&data_folder.join("settings.json")).is_err(), "{identifier}");
            }
        }

        #[test]
        fn a_feature_that_is_not_a_plain_name_is_refused() {
            let fixtures = tempfile::tempdir().unwrap();

            for feature in ["..", ".", ""] {
                assert!(case_files(fixtures.path(), feature).is_err(), "{feature}");
            }
        }

        #[test]
        fn a_missing_case_file_given_by_its_path_is_refused_naming_the_path() {
            let fixtures = tempfile::tempdir().unwrap();
            let absolute = fixtures.path().join("demos").join("gone.toml");
            let absolute = absolute.to_str().unwrap();

            for target in [absolute, "no-such-folder/world.toml", "no-such-world.toml"] {
                assert!(case_files(fixtures.path(), target).unwrap_err().contains(target), "{target}");
            }
        }

        #[test]
        fn a_missing_or_invalid_case_file_is_reported_by_its_path() {
            let fixtures = tempfile::tempdir().unwrap();
            fs::write(fixtures.path().join("bad.toml"), "colour = 1\n").unwrap();

            assert!(case_files(fixtures.path(), "trial").unwrap_err().contains("trial.toml"));
            assert!(case_files(fixtures.path(), "bad").unwrap_err().contains("bad.toml"));
            let empty = tempfile::tempdir().unwrap();
            assert!(case_files(empty.path(), "all").unwrap_err().contains("no case file"));
        }
    }

    mod positive {
        use std::fs;
        use std::path::PathBuf;

        use super::{CASE_FILE, args};
        use crate::{Command, case_files, folder_name, parse_args};

        /// Every file of `fixtures/demos/` is a demo world with scenery cases only.
        #[test]
        fn every_demo_world_of_the_repository_parses() {
            let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/demos");
            let mut worlds = 0;
            for entry in fs::read_dir(&folder).unwrap() {
                let path = entry.unwrap().path();
                if path.extension().is_none_or(|extension| extension != "toml") {
                    continue;
                }
                let file = crate::CaseFile::parse(&fs::read_to_string(&path).unwrap())
                    .map_err(|error| format!("{}: {error}", path.display()))
                    .unwrap();
                assert!(file.demo().is_some(), "{} has no [demo] header", path.display());
                worlds += 1;
            }
            assert!(worlds > 0, "no demo world in {}", folder.display());
        }

        #[test]
        fn reads_the_feature_and_the_folder() {
            let into = parse_args(args(&["generate", "all", "--into", "/tmp/x"])).unwrap();
            let default = parse_args(args(&["generate", "trial"])).unwrap();

            assert_eq!(into, Command::Generate { target: "all".into(), into: Some(PathBuf::from("/tmp/x")) });
            assert_eq!(default, Command::Generate { target: "trial".into(), into: None });
        }

        #[test]
        fn reads_the_scenario_and_the_folder_of_a_demo() {
            let into = parse_args(args(&["demo", "overview", "--into", "/tmp/x"])).unwrap();
            let default = parse_args(args(&["demo", "overview"])).unwrap();

            assert_eq!(into, Command::Demo { scenario: "overview".into(), into: Some(PathBuf::from("/tmp/x")) });
            assert_eq!(default, Command::Demo { scenario: "overview".into(), into: None });
        }

        #[test]
        fn keeps_a_path_target_as_given() {
            for target in ["fixtures/demos/developer-mac.toml", "../worlds/./mac.toml", "world.toml"] {
                let command = parse_args(args(&["generate", target])).unwrap();

                assert_eq!(command, Command::Generate { target: target.into(), into: None }, "{target}");
            }
        }

        #[test]
        fn a_path_target_is_read_by_its_path_into_a_folder_named_by_its_stem() {
            let folder = tempfile::tempdir().unwrap();
            let world = folder.path().join("demos").join("developer-mac.toml");
            fs::create_dir(folder.path().join("demos")).unwrap();
            fs::write(&world, CASE_FILE.replace("trial", "developer-mac")).unwrap();
            let target = world.to_str().unwrap();

            let files = case_files(&folder.path().join("sandboxes"), target).unwrap();

            assert_eq!(files.iter().map(fresher_sandbox::CaseFile::feature).collect::<Vec<_>>(), ["developer-mac"]);
            assert_eq!(folder_name(target), "developer-mac");
            assert_eq!(folder_name("world.toml"), "world");
            assert_eq!(folder_name("trial"), "trial");
            assert_eq!(folder_name("all"), "all");
        }

        #[test]
        fn all_reads_every_case_file_by_name() {
            let fixtures = tempfile::tempdir().unwrap();
            fs::write(fixtures.path().join("trial.toml"), CASE_FILE).unwrap();
            fs::write(fixtures.path().join("guards.toml"), CASE_FILE.replace("trial", "guards")).unwrap();
            fs::write(fixtures.path().join("notes.md"), "not a case file").unwrap();

            let all = case_files(fixtures.path(), "all").unwrap();
            let one = case_files(fixtures.path(), "trial").unwrap();

            let features: Vec<_> = all.iter().map(fresher_sandbox::CaseFile::feature).collect();
            assert_eq!(features, ["guards", "trial"]);
            assert_eq!(one[0].feature(), "trial");
        }

        /// `trial-off.toml` sorts before `trial.toml` by full name; by stem the base file comes first.
        #[test]
        fn all_puts_a_base_file_before_its_variants() {
            let fixtures = tempfile::tempdir().unwrap();
            for feature in ["trial-off", "trial", "trial-denied"] {
                fs::write(fixtures.path().join(format!("{feature}.toml")), CASE_FILE.replace("trial", feature))
                    .unwrap();
            }

            let all = case_files(fixtures.path(), "all").unwrap();

            let features: Vec<_> = all.iter().map(fresher_sandbox::CaseFile::feature).collect();
            assert_eq!(features, ["trial", "trial-denied", "trial-off"]);
        }
    }
}
