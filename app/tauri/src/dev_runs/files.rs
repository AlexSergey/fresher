//! The record's files: a pair created new (never overwriting), events appended, the snapshot rewritten atomically.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::Serialize;

use super::{BuildName, DevRunsRefusal, Mode, clock};

const LAST_ATTEMPT: u32 = 99;

/// The stem both files of a run share: `<stamp>_<build>_<mode>`, with `-<attempt>` appended from the second.
pub(crate) fn file_stem(started: SystemTime, build: &BuildName, mode: &Mode, attempt: u32) -> String {
    let stem = format!("{}_{}_{}", clock::utc(started).stamp(), build.name(), mode.label());
    if attempt <= 1 { stem } else { format!("{stem}-{attempt}") }
}

/// The two files of one run: the snapshot (replaced whole) and the events (appended).
#[derive(Debug)]
pub(crate) struct RunFiles {
    folder: PathBuf,
    snapshot: PathBuf,
    events: File,
}

impl RunFiles {
    /// Creates the folder and a new pair in it. A name either file of which exists is skipped, so nothing that
    /// existed is ever opened; after attempt 99 the run is refused. A file a lost race left empty stays.
    pub(crate) fn create(
        folder: &Path,
        started: SystemTime,
        build: &BuildName,
        mode: &Mode,
    ) -> Result<Self, DevRunsRefusal> {
        fs::create_dir_all(folder).map_err(|source| DevRunsRefusal::Folder { path: folder.to_owned(), source })?;
        let is_folder = fs::symlink_metadata(folder).is_ok_and(|meta| meta.is_dir());
        if !is_folder {
            return Err(DevRunsRefusal::NotAFolder { path: folder.to_owned() });
        }
        for attempt in 1..=LAST_ATTEMPT {
            let stem = file_stem(started, build, mode, attempt);
            let snapshot = folder.join(format!("{stem}.json"));
            let events_path = folder.join(format!("{stem}.events.jsonl"));
            if fs::symlink_metadata(&snapshot).is_ok() || fs::symlink_metadata(&events_path).is_ok() {
                continue;
            }
            match create_new(&snapshot, false) {
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(source) => return Err(DevRunsRefusal::Write { path: snapshot, source }),
            }
            match create_new(&events_path, true) {
                Ok(events) => return Ok(Self { folder: folder.to_owned(), snapshot, events }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(source) => return Err(DevRunsRefusal::Write { path: events_path, source }),
            }
        }
        Err(DevRunsRefusal::Taken { stem: file_stem(started, build, mode, 1) })
    }

    /// Appends one compact JSON line, written at once, and flushes it.
    pub(crate) fn append(&mut self, line: &impl Serialize) -> io::Result<()> {
        let mut bytes = serde_json::to_vec(line)?;
        bytes.push(b'\n');
        self.events.write_all(&bytes)?;
        self.events.flush()
    }

    /// Replaces the snapshot whole with `record`, pretty-printed ([`replace`]).
    pub(crate) fn rewrite(&self, record: &impl Serialize) -> io::Result<()> {
        replace(&self.folder, &self.snapshot, &pretty(record)?)
    }

    pub(crate) fn folder(&self) -> &Path {
        &self.folder
    }

    pub(crate) fn snapshot(&self) -> &Path {
        &self.snapshot
    }
}

/// `record` as the snapshot holds it: pretty-printed, ending with a new line.
pub(crate) fn pretty(record: &impl Serialize) -> io::Result<Vec<u8>> {
    let mut bytes = serde_json::to_vec_pretty(record)?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// Replaces `snapshot` whole with `bytes`: a temporary file in `folder` (the snapshot's own), synced, renamed over it.
pub(crate) fn replace(folder: &Path, snapshot: &Path, bytes: &[u8]) -> io::Result<()> {
    let stem = snapshot.file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or_default();
    let mut temporary = tempfile::Builder::new().prefix(&format!(".{stem}.")).suffix(".tmp").tempfile_in(folder)?;
    temporary.write_all(bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist(snapshot).map_err(|error| error.error)?;
    Ok(())
}

fn create_new(path: &Path, append: bool) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    if append {
        options.append(true);
    }
    options.open(path)
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::time::SystemTime;

    use crate::dev_runs::BuildName;
    use crate::dev_runs::clock::at_secs;

    const FIRST: &str = "2025-11-20_10-04-31_0.1.0+26e4cb0-dirty_mac";

    fn started() -> SystemTime {
        at_secs(1_763_633_071)
    }

    fn build() -> BuildName {
        BuildName::new("0.1.0", Some("26e4cb0".as_ref()), Some("1".as_ref()))
    }

    fn names(folder: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    mod negative {
        use std::fs;

        use super::{FIRST, build, names, started};
        use crate::dev_runs::files::{RunFiles, file_stem};
        use crate::dev_runs::{DevRunsRefusal, Mode};

        #[test]
        fn an_existing_record_is_never_opened() {
            let folder = tempfile::tempdir().unwrap();
            let old = folder.path().join(format!("{FIRST}.json"));
            fs::write(&old, b"old bytes").unwrap();

            let files = RunFiles::create(folder.path(), started(), &build(), &Mode::Mac).unwrap();

            assert!(files.snapshot().ends_with(format!("{FIRST}-2.json")));
            assert_eq!(fs::read(&old).unwrap(), b"old bytes");
        }

        #[test]
        fn an_existing_events_file_moves_to_the_next_name() {
            let folder = tempfile::tempdir().unwrap();
            let old = folder.path().join(format!("{FIRST}.events.jsonl"));
            fs::write(&old, b"old line\n").unwrap();

            let files = RunFiles::create(folder.path(), started(), &build(), &Mode::Mac).unwrap();

            assert!(files.snapshot().ends_with(format!("{FIRST}-2.json")));
            assert_eq!(fs::read(&old).unwrap(), b"old line\n");
            assert!(!folder.path().join(format!("{FIRST}.json")).exists());
        }

        #[test]
        fn ninety_nine_taken_names_refuse() {
            let folder = tempfile::tempdir().unwrap();
            for attempt in 1..=99 {
                let stem = file_stem(started(), &build(), &Mode::Mac, attempt);
                fs::write(folder.path().join(format!("{stem}.json")), b"x").unwrap();
            }
            let before = names(folder.path());

            let result = RunFiles::create(folder.path(), started(), &build(), &Mode::Mac);

            assert!(matches!(result, Err(DevRunsRefusal::Taken { .. })));
            assert_eq!(names(folder.path()), before);
        }

        #[test]
        fn a_folder_that_is_a_link_is_refused() {
            let root = tempfile::tempdir().unwrap();
            let target = root.path().join("target");
            fs::create_dir(&target).unwrap();
            let link = root.path().join("link");
            std::os::unix::fs::symlink(&target, &link).unwrap();

            let result = RunFiles::create(&link, started(), &build(), &Mode::Mac);

            assert!(matches!(result, Err(DevRunsRefusal::NotAFolder { .. })));
            assert_eq!(names(&target), Vec::<String>::new());
        }

        #[test]
        fn a_file_in_place_of_the_folder_is_refused() {
            let root = tempfile::tempdir().unwrap();
            let file = root.path().join("not-a-folder");
            fs::write(&file, b"x").unwrap();

            let result = RunFiles::create(&file, started(), &build(), &Mode::Mac);

            assert!(matches!(result, Err(DevRunsRefusal::Folder { .. } | DevRunsRefusal::NotAFolder { .. })));
            assert_eq!(fs::read(&file).unwrap(), b"x");
        }

        #[test]
        fn a_rewrite_leaves_no_temporary_file_and_touches_nothing_else() {
            let folder = tempfile::tempdir().unwrap();
            let other = folder.path().join("other.json");
            fs::write(&other, b"other bytes").unwrap();
            let files = RunFiles::create(folder.path(), started(), &build(), &Mode::Mac).unwrap();

            files.rewrite(&serde_json::json!({ "a": 1 })).unwrap();

            assert_eq!(fs::read(&other).unwrap(), b"other bytes");
            assert!(names(folder.path()).iter().all(|name| !name.contains(".tmp")));
            assert_eq!(names(folder.path()).len(), 3);
        }
    }

    mod positive {
        use std::fs;

        use serde_json::json;

        use super::{FIRST, build, names, started};
        use crate::dev_runs::Mode;
        use crate::dev_runs::files::RunFiles;

        #[test]
        fn the_first_pair_is_named_by_time_build_and_mode() {
            let folder = tempfile::tempdir().unwrap();

            RunFiles::create(folder.path(), started(), &build(), &Mode::Mac).unwrap();

            assert_eq!(names(folder.path()), [format!("{FIRST}.events.jsonl"), format!("{FIRST}.json")]);
        }

        #[test]
        fn the_folder_is_created() {
            let root = tempfile::tempdir().unwrap();
            let folder = root.path().join("a").join("dev-runs");

            RunFiles::create(&folder, started(), &build(), &Mode::Sandbox("trial".into())).unwrap();

            assert!(folder.join("2025-11-20_10-04-31_0.1.0+26e4cb0-dirty_sandbox-trial.json").is_file());
        }

        #[test]
        fn events_are_appended_one_line_each() {
            let folder = tempfile::tempdir().unwrap();
            let mut files = RunFiles::create(folder.path(), started(), &build(), &Mode::Mac).unwrap();

            for n in 1..=3 {
                files.append(&json!({ "n": n })).unwrap();
            }

            let text = fs::read_to_string(folder.path().join(format!("{FIRST}.events.jsonl"))).unwrap();
            let lines: Vec<serde_json::Value> = text.lines().map(|line| serde_json::from_str(line).unwrap()).collect();
            assert_eq!(lines, [json!({ "n": 1 }), json!({ "n": 2 }), json!({ "n": 3 })]);
        }

        #[test]
        fn the_snapshot_is_replaced_whole() {
            let folder = tempfile::tempdir().unwrap();
            let files = RunFiles::create(folder.path(), started(), &build(), &Mode::Mac).unwrap();

            files.rewrite(&json!({ "first": "a long first record" })).unwrap();
            files.rewrite(&json!({ "second": 2 })).unwrap();

            let parsed: serde_json::Value = serde_json::from_slice(&fs::read(files.snapshot()).unwrap()).unwrap();
            assert_eq!(parsed, json!({ "second": 2 }));
        }
    }
}
