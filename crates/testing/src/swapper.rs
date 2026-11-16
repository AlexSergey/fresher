//! A folder swapped for a link and back, again and again, on a thread of its own: the race a cleaner must survive
//! between its check of an item and its act on it.

use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// How long the folder stays in its place between two swaps.
const REAL_PHASE: Duration = Duration::from_micros(200);

/// Swaps a folder for a link to another folder and back until dropped, or until a rename fails (the code under test
/// removed one of the names). It only renames: it never removes anything itself. The link waits beside the folder
/// with the extension `link`, the folder while it is swapped out with `away`.
#[derive(Debug)]
pub struct Swapper {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Swapper {
    /// Starts swapping `folder` for a link to `target`; returns once the first swap was tried.
    pub fn start(folder: &Path, target: &Path) -> io::Result<Self> {
        let (folder, away, link) = (folder.to_path_buf(), folder.with_extension("away"), folder.with_extension("link"));
        if fs::symlink_metadata(&link).is_err() {
            symlink(target, &link)?;
        }
        let (stop, started) = (Arc::new(AtomicBool::new(false)), Arc::new(AtomicBool::new(false)));
        let (stopped, starting) = (Arc::clone(&stop), Arc::clone(&started));
        let thread = thread::spawn(move || {
            while !stopped.load(Ordering::Relaxed) {
                let swapped = fs::rename(&folder, &away).and_then(|()| fs::rename(&link, &folder));
                starting.store(true, Ordering::Relaxed);
                thread::yield_now();
                let back = fs::rename(&folder, &link).and_then(|()| fs::rename(&away, &folder));
                if swapped.and(back).is_err() {
                    break;
                }
                // The folder stays in place a little longer than the link, so a check through it can pass and the act
                // after it can meet the link.
                thread::sleep(REAL_PHASE);
            }
            starting.store(true, Ordering::Relaxed);
        });
        while !started.load(Ordering::Relaxed) {
            thread::yield_now();
        }
        Ok(Self { stop, thread: Some(thread) })
    }
}

impl Drop for Swapper {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::fs;

        use crate::Swapper;

        #[test]
        fn a_missing_folder_stops_the_swapping_and_nothing_is_removed() {
            let root = tempfile::tempdir().unwrap();
            fs::create_dir(root.path().join("target")).unwrap();

            drop(Swapper::start(&root.path().join("missing"), &root.path().join("target")).unwrap());

            assert!(root.path().join("target").is_dir());
            assert!(fs::symlink_metadata(root.path().join("missing.link")).unwrap().file_type().is_symlink());
        }
    }

    mod positive {
        use std::fs;

        use crate::Swapper;

        #[test]
        fn the_folder_is_back_in_its_place_when_dropped() {
            let root = tempfile::tempdir().unwrap();
            for folder in ["folder", "target"] {
                fs::create_dir(root.path().join(folder)).unwrap();
            }
            fs::write(root.path().join("folder/kept"), b"x").unwrap();

            drop(Swapper::start(&root.path().join("folder"), &root.path().join("target")).unwrap());

            assert!(root.path().join("folder/kept").exists());
            assert!(!root.path().join("folder.away").exists());
        }
    }
}
