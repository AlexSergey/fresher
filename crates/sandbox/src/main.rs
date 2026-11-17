//! `fresher-sandbox generate <feature|all> [--into <dir>]`: writes the case files of a feature
//! (`fixtures/sandboxes/<feature>.toml`, every file for `all`) into a sandbox folder, `sandbox/<feature>/` at the
//! repository's root by default. Dev only: `npm run sandbox -- <feature>`.

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;
use std::time::SystemTime;

use fresher_core::{Protection, ProtectionError};
use fresher_sandbox::{CaseFile, generate};

const USAGE: &str = "usage: fresher-sandbox generate <feature|all> [--into <dir>]";
/// The identifiers of the release app and of the development build: both data folders are protected like the app's
/// own.
const APP_IDENTIFIERS: [&str; 2] = ["io.github.alexsergey.fresher", "io.github.alexsergey.fresher.dev"];

/// What to generate, read from the arguments.
#[derive(Debug, PartialEq, Eq)]
struct Command {
    feature: String,
    into: Option<PathBuf>,
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
    let files = case_files(&repository.join("fixtures").join("sandboxes"), &command.feature)?;
    let into = command.into.unwrap_or_else(|| repository.join("sandbox").join(&command.feature));
    // The binary's one home lookup: only for the real Mac's protection list, which keeps the sandbox off the home.
    #[expect(clippy::disallowed_methods, reason = "the home folder is looked up once per binary, here")]
    let home = std::env::home_dir().ok_or("the home folder is unknown")?;
    let protection = real_protection(&home).map_err(|error| error.to_string())?;
    let generated = generate(&files, &into, SystemTime::now(), &protection, fresher_macos::current_user())
        .map_err(|error| error.to_string())?;
    report(&files, generated.sandbox.path(), &generated.strangers);
    Ok(())
}

/// The real Mac's protection list for `home`, with the data folders of both the release app and the development
/// build.
fn real_protection(home: &Path) -> Result<Protection, ProtectionError> {
    let [release, development] = APP_IDENTIFIERS.map(|identifier| {
        let data_folder = home.join("Library").join("Application Support").join(identifier);
        fresher_macos::protection(home, &data_folder, None)
    });
    release?.joined(&development?)
}

/// Reads `generate <feature|all> [--into <dir>]`.
fn parse_args(mut args: impl Iterator<Item = OsString>) -> Result<Command, String> {
    if args.next().as_deref() != Some("generate".as_ref()) {
        return Err(USAGE.into());
    }
    let feature = args.next().and_then(|feature| feature.into_string().ok()).ok_or(USAGE)?;
    let into = match (args.next(), args.next(), args.next()) {
        (None, ..) => None,
        (Some(flag), Some(dir), None) if flag == "--into" => Some(PathBuf::from(dir)),
        _ => return Err(USAGE.into()),
    };
    Ok(Command { feature, into })
}

/// The case files of `feature` in `fixtures`, or every one of them, by name, for `all`.
fn case_files(fixtures: &Path, feature: &str) -> Result<Vec<CaseFile>, String> {
    let paths = if feature == "all" {
        let entries = fs::read_dir(fixtures).map_err(|error| format!("{}: {error}", fixtures.display()))?;
        let mut paths: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "toml"))
            .collect();
        paths.sort();
        paths
    } else {
        let mut name = Path::new(feature).components();
        if !matches!((name.next(), name.next()), (Some(Component::Normal(_)), None)) {
            return Err(format!("`{feature}` is not a feature name"));
        }
        vec![fixtures.join(format!("{feature}.toml"))]
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

            for feature in ["../trial", "a/b", "/etc/hosts", ""] {
                assert!(case_files(fixtures.path(), feature).is_err(), "{feature}");
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
        use crate::{Command, case_files, parse_args};

        #[test]
        fn reads_the_feature_and_the_folder() {
            let into = parse_args(args(&["generate", "all", "--into", "/tmp/x"])).unwrap();
            let default = parse_args(args(&["generate", "trial"])).unwrap();

            assert_eq!(into, Command { feature: "all".into(), into: Some(PathBuf::from("/tmp/x")) });
            assert_eq!(default, Command { feature: "trial".into(), into: None });
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
    }
}
