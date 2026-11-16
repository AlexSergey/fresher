//! `MacOpenFiles`: what the user's own processes hold open, from the process table.

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use fresher_core::{OpenFiles, OpenFilesError, OpenSet};
use libc::pid_t;

use crate::ffi::libproc;

/// How long one walk of the mapped regions is reused: it takes up to a second, the descriptors milliseconds (design,
/// call 38).
const MAPPED_REUSE: Duration = Duration::from_secs(30);

/// The files and folders the current user's processes hold: every vnode descriptor, every working folder and every
/// mapped file. This process is left out: the cleaner itself holds the item it checks. Processes of other users
/// cannot be seen without administrator rights (they answer `EPERM`) and are counted, not failed.
#[derive(Debug, Default)]
pub struct MacOpenFiles {
    mapped: Mutex<Option<MappedWalk>>,
}

/// One walk of the mapped regions, kept for `MAPPED_REUSE` for the same `within`.
#[derive(Debug)]
struct MappedWalk {
    within: Vec<PathBuf>,
    taken: Instant,
    held: Vec<(PathBuf, String)>,
}

impl MacOpenFiles {
    pub fn new() -> Self {
        Self::default()
    }

    /// The mapped files under `within`, from the last walk while it is fresh, else from a new one.
    fn mapped(&self, pids: &[pid_t], within: &[PathBuf], names: &mut Names) -> Vec<(PathBuf, String)> {
        let mut walk = self.mapped.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(walk) = walk.as_ref().filter(|walk| walk.within == within && walk.taken.elapsed() < MAPPED_REUSE) {
            return walk.held.clone();
        }
        let taken = Instant::now();
        let mut held = Vec::new();
        for &pid in pids {
            match libproc::mapped_files(pid) {
                Ok(files) => held.extend(kept(files, within).map(|file| (file, names.of(pid)))),
                Err(error) => tracing::debug!(pid, %error, "the mapped files of a process are unknown"),
            }
        }
        *walk = Some(MappedWalk { within: within.to_vec(), taken, held: held.clone() });
        held
    }
}

impl OpenFiles for MacOpenFiles {
    fn snapshot(&self, within: &[PathBuf]) -> Result<OpenSet, OpenFilesError> {
        let pids = user_processes().map_err(|error| OpenFilesError { reason: error.to_string() })?;
        let mut names = Names::default();
        let mut held = Vec::new();
        let mut withheld = 0_usize;
        for &pid in &pids {
            let mut paths = Vec::new();
            match libproc::vnode_descriptors(pid) {
                Ok(descriptors) => {
                    for fd in descriptors {
                        match libproc::descriptor_path(pid, fd) {
                            Ok(path) => paths.extend(path),
                            Err(error) if error.raw_os_error() == Some(libc::EPERM) => withheld += 1,
                            Err(_) => {}
                        }
                    }
                }
                Err(error) => tracing::debug!(pid, %error, "the descriptors of a process are unknown"),
            }
            paths.extend(libproc::working_folder(pid).ok().flatten());
            held.extend(kept(paths, within).map(|path| (path, names.of(pid))));
        }
        held.extend(self.mapped(&pids, within, &mut names));
        tracing::debug!(processes = pids.len(), withheld, held = held.len(), "open files");
        Ok(OpenSet::new(held))
    }
}

/// The current user's processes but this one. Another user's process answers `EPERM` and a process that ended
/// `ESRCH`: both are left out. Only a failure to list the processes at all is an error.
fn user_processes() -> io::Result<Vec<pid_t>> {
    let user = libproc::user_id();
    let this = pid_t::try_from(std::process::id()).unwrap_or(0);
    let mut others = 0_usize;
    let mut pids = Vec::new();
    for pid in libproc::all_pids()? {
        if pid <= 0 || pid == this {
            continue;
        }
        match libproc::owner(pid) {
            Ok(owner) if owner == user => pids.push(pid),
            Ok(_) => others += 1,
            Err(error) if error.raw_os_error() == Some(libc::EPERM) => others += 1,
            Err(_) => {}
        }
    }
    tracing::debug!(mine = pids.len(), others, "processes");
    Ok(pids)
}

/// The paths at or under one of `within`, by components: `/a/cache2` is not under `/a/cache`.
fn kept(paths: Vec<PathBuf>, within: &[PathBuf]) -> impl Iterator<Item = PathBuf> + '_ {
    paths.into_iter().filter(move |path| within.iter().any(|place| path.starts_with(place)))
}

/// The name of each process, looked up once and only for a process that holds something.
#[derive(Debug, Default)]
struct Names(HashMap<pid_t, String>);

impl Names {
    fn of(&mut self, pid: pid_t) -> String {
        self.0.entry(pid).or_insert_with(|| process_name(pid)).clone()
    }
}

/// The outermost app bundle's name in the executable's path, else the process's short name: a helper's short name is
/// not its app's (design, call 35).
fn process_name(pid: pid_t) -> String {
    libproc::executable(pid)
        .as_deref()
        .and_then(app_name)
        .or_else(|| libproc::short_name(pid))
        .unwrap_or_else(|| format!("process {pid}"))
}

fn app_name(executable: &Path) -> Option<String> {
    executable
        .ancestors()
        .filter(|folder| folder.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("app")))
        .last()
        .and_then(Path::file_stem)
        .map(|name| name.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, Stdio};
    use std::time::{Duration, Instant};

    use fresher_core::{OpenFiles as _, OpenSet};

    use crate::MacOpenFiles;

    /// The variable that makes `map_a_file_and_wait` map a file: the path of that file.
    const MAP_FILE: &str = "FRESHER_TEST_MAP_FILE";
    /// How long a test waits for its child to hold what it was told to.
    const PATIENCE: Duration = Duration::from_secs(10);

    /// A canonical temporary folder: the kernel names paths as `/private/var/…`, never through `/var`.
    fn folder() -> (tempfile::TempDir, PathBuf) {
        let folder = tempfile::tempdir().unwrap();
        let path = fs::canonicalize(folder.path()).unwrap();
        (folder, path)
    }

    /// A child process, killed when dropped.
    struct Holder(Child);

    impl Drop for Holder {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    /// A shell that opens `file` (when given), goes into `working`, then becomes `sleep`.
    fn hold(file: Option<&Path>, working: &Path) -> Holder {
        let script = r#"[ -n "$1" ] && exec 3< "$1"; cd "$2" && exec /bin/sleep 60"#;
        let file = file.map_or_else(Default::default, |file| file.as_os_str().to_owned());
        let child = Command::new("/bin/sh")
            .args(["-c".as_ref(), script.as_ref(), "sh".as_ref(), file.as_os_str(), working.as_os_str()])
            .stdin(Stdio::null())
            .spawn()
            .unwrap();
        Holder(child)
    }

    /// Snapshots until `path` has a holder, or gives the last snapshot after `PATIENCE`.
    fn until_held(open_files: &MacOpenFiles, within: &[PathBuf], path: &Path) -> OpenSet {
        let start = Instant::now();
        loop {
            let set = open_files.snapshot(within).unwrap();
            if set.holder_under(path).is_some() || start.elapsed() > PATIENCE {
                return set;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    mod negative {
        use std::fs::{self, File};
        use std::path::Path;

        use fresher_core::OpenFiles as _;

        use super::{folder, hold, until_held};
        use crate::MacOpenFiles;
        use crate::open_files::app_name;

        #[test]
        fn a_process_outside_an_app_bundle_has_no_app_name() {
            assert_eq!(app_name(Path::new("/bin/sleep")), None);
            assert_eq!(app_name(Path::new("/opt/tools/app/bin/tool")), None);
        }

        #[test]
        fn a_file_nobody_holds_is_not_in_the_set() {
            let (_folder, root) = folder();
            let file = root.join("nobody.txt");
            fs::write(&file, b"held by nobody").unwrap();

            let set = MacOpenFiles::new().snapshot(std::slice::from_ref(&root)).unwrap();

            assert_eq!(set.holder_under(&file), None);
        }

        #[test]
        fn this_process_is_never_a_holder() {
            let (_folder, root) = folder();
            let file = root.join("checked.txt");
            fs::write(&file, b"the cleaner holds what it checks").unwrap();
            let _held = File::open(&file).unwrap();

            let set = MacOpenFiles::new().snapshot(std::slice::from_ref(&root)).unwrap();

            assert_eq!(set.holder_under(&file), None);
        }

        #[test]
        fn a_file_outside_within_is_not_kept() {
            let (_folder, root) = folder();
            let (held, other) = (root.join("held"), root.join("other"));
            fs::create_dir(&held).unwrap();
            fs::create_dir(&other).unwrap();
            let file = held.join("file.txt");
            fs::write(&file, b"held").unwrap();
            let _holder = hold(Some(&file), &root);
            let open_files = MacOpenFiles::new();
            assert!(until_held(&open_files, std::slice::from_ref(&held), &file).holder_under(&file).is_some());

            let set = open_files.snapshot(std::slice::from_ref(&other)).unwrap();

            assert_eq!(set.holder_under(&file), None);
            assert!(set.is_empty());
        }

        #[test]
        fn a_shared_prefix_is_not_inside() {
            let (_folder, root) = folder();
            let (cache, cache2) = (root.join("a").join("cache"), root.join("a").join("cache2"));
            fs::create_dir_all(&cache).unwrap();
            fs::create_dir_all(&cache2).unwrap();
            let _holder = hold(None, &cache);
            let open_files = MacOpenFiles::new();
            assert!(until_held(&open_files, std::slice::from_ref(&root), &cache).holder_under(&cache).is_some());

            let set = open_files.snapshot(std::slice::from_ref(&cache2)).unwrap();

            assert_eq!(set.holder_under(&cache2), None);
            assert!(set.is_empty());
        }
    }

    mod positive {
        use std::fs::{self, File};
        use std::io::Read as _;
        use std::path::Path;
        use std::process::{Command, Stdio};
        use std::time::{Duration, Instant};

        use fresher_core::OpenFiles as _;

        use super::{Holder, MAP_FILE, PATIENCE, folder, hold, until_held};
        use crate::MacOpenFiles;
        use crate::ffi::libproc::Mapping;
        use crate::open_files::app_name;

        #[test]
        fn the_name_is_the_outermost_app_bundles() {
            let helper = "/Applications/Big Browser.app/Contents/Frameworks/Big Browser Framework.framework/Helpers/\
                          Big Browser Helper (Renderer).app/Contents/MacOS/Big Browser Helper (Renderer)";

            assert_eq!(app_name(Path::new(helper)).as_deref(), Some("Big Browser"));
            assert_eq!(app_name(Path::new("/Applications/Notes.APP/Contents/MacOS/Notes")).as_deref(), Some("Notes"));
        }

        #[test]
        fn a_file_held_by_a_child_process_is_found_with_its_name() {
            let (_folder, root) = folder();
            let file = root.join("held.txt");
            fs::write(&file, b"held open by a child").unwrap();
            let _holder = hold(Some(&file), &root);

            let set = until_held(&MacOpenFiles::new(), std::slice::from_ref(&root), &file);

            assert_eq!(set.holder_under(&file), Some("sleep"));
        }

        #[test]
        fn a_working_folder_counts_as_held() {
            let (_folder, root) = folder();
            let working = root.join("working");
            fs::create_dir(&working).unwrap();
            let _holder = hold(None, &working);

            let set = until_held(&MacOpenFiles::new(), std::slice::from_ref(&root), &working);

            assert_eq!(set.holder_under(&working), Some("sleep"));
        }

        #[test]
        fn a_file_mapped_without_a_descriptor_is_held() {
            let (_folder, root) = folder();
            let file = root.join("mapped.db");
            fs::write(&file, vec![7_u8; 64 * 1024]).unwrap();
            let ready = root.join("mapped.ready");
            let child = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "open_files::tests::positive::map_a_file_and_wait", "--ignored", "--quiet"])
                .env(MAP_FILE, &file)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .spawn()
                .unwrap();
            let mut holder = Holder(child);
            let start = Instant::now();
            while !ready.exists() && start.elapsed() < PATIENCE {
                std::thread::sleep(Duration::from_millis(20));
            }

            let set = MacOpenFiles::new().snapshot(std::slice::from_ref(&root)).unwrap();

            assert!(ready.exists(), "the child did not map the file");
            assert!(set.holder_under(&file).is_some());
            drop(holder.0.stdin.take());
            assert!(holder.0.wait().unwrap().success());
        }

        /// Run only as the child of `a_file_mapped_without_a_descriptor_is_held`: maps the file the variable names,
        /// closes its descriptor, says so with a file beside it, and keeps the mapping until its input closes.
        #[test]
        #[ignore = "the child of a_file_mapped_without_a_descriptor_is_held"]
        fn map_a_file_and_wait() {
            let Some(path) = std::env::var_os(MAP_FILE) else { return };
            let path = Path::new(&path);
            let file = File::open(path).unwrap();
            let mapping = Mapping::of(&file, 64 * 1024).unwrap();
            drop(file);
            fs::write(path.with_extension("ready"), b"mapped").unwrap();

            std::io::stdin().read_to_end(&mut Vec::new()).unwrap();

            drop(mapping);
        }
    }
}
