//! Facts about the running process, and the other processes of the system.

use std::io;
use std::path::PathBuf;

use fresher_core::ByteSize;
use libc::pid_t;

use crate::ffi::{libproc, rusage};

/// A process of the system, as the process table names it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Process {
    pub pid: i32,
    /// Its short name, if the system gives one.
    pub name: Option<String>,
    /// Its executable's path, if the system gives one (it may not for another user's process).
    pub executable: Option<PathBuf>,
}

/// Every process of the system but this one, with what the process table tells of it: only reads. A process that
/// ended meanwhile is left out; only a failure to list the processes at all is an error.
pub fn running_processes() -> io::Result<Vec<Process>> {
    let this = pid_t::try_from(std::process::id()).unwrap_or(0);
    let processes: Vec<Process> = libproc::all_pids()?
        .into_iter()
        .filter(|&pid| pid > 0 && pid != this)
        .map(|pid| Process { pid, name: libproc::short_name(pid), executable: libproc::executable(pid) })
        .filter(|process| process.name.is_some() || process.executable.is_some())
        .collect();
    tracing::debug!(processes = processes.len(), "running processes");
    Ok(processes)
}

/// The user this process runs as (`getuid`): the owner the cleaner compares an item's with.
pub fn current_user() -> u32 {
    libproc::user_id()
}

/// The largest resident memory the process has used so far; zero if the system does not tell.
pub fn peak_memory() -> ByteSize {
    match rusage::peak_resident_bytes() {
        Ok(bytes) => ByteSize::new(u64::try_from(bytes).unwrap_or(0)),
        Err(error) => {
            tracing::warn!(%error, "the peak memory is unknown");
            ByteSize::ZERO
        }
    }
}

#[cfg(test)]
mod tests {
    mod positive {
        use std::hint::black_box;
        use std::os::unix::fs::MetadataExt as _;
        use std::process::Command;

        use crate::{current_user, peak_memory, running_processes};

        #[test]
        fn the_running_processes_hold_a_child_and_not_this_process() {
            let mut child = Command::new("/bin/sleep").arg("5").spawn().unwrap();
            let pid = i32::try_from(child.id()).unwrap();

            let processes = running_processes();

            child.kill().unwrap();
            child.wait().unwrap();
            let processes = processes.unwrap();
            let sleeping = processes.iter().find(|process| process.pid == pid).unwrap();
            assert_eq!(sleeping.executable.as_deref(), Some(std::path::Path::new("/bin/sleep")));
            assert_eq!(sleeping.name.as_deref(), Some("sleep"));
            let this = i32::try_from(std::process::id()).unwrap();
            assert!(processes.iter().all(|process| process.pid != this));
        }

        #[test]
        fn the_current_user_owns_a_new_temporary_file() {
            let file = tempfile::NamedTempFile::new().unwrap();

            assert_eq!(file.as_file().metadata().unwrap().uid(), current_user());
        }

        #[test]
        fn peak_memory_grows_after_a_large_allocation() {
            const SIZE: usize = 256 * 1024 * 1024;
            let before = peak_memory();

            let touched = black_box(vec![1_u8; SIZE]);
            let after = peak_memory();
            drop(touched);

            assert!(before.bytes() > 0);
            assert!(after.bytes() >= before.bytes() + (SIZE as u64) / 2, "{before} -> {after}");
        }
    }
}
