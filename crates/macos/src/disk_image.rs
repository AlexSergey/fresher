//! A disk image the test creates and mounts in its own temporary folder, detached when dropped (tests only).

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

const HDIUTIL: &str = "/usr/bin/hdiutil";

/// A file system `hdiutil create -fs` knows.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Format {
    Apfs,
    Fat,
}

/// A mounted disk image. It is mounted with `-nobrowse`, so Finder does not show it (and the system keeps no change
/// journal for it); it never touches another volume.
#[derive(Debug)]
pub(crate) struct DiskImage {
    mount_point: PathBuf,
    _folder: TempDir,
}

impl DiskImage {
    /// `None` when `hdiutil` is not on this Mac: the test that needs it is skipped.
    pub(crate) fn mount(format: Format) -> Option<Self> {
        if !Path::new(HDIUTIL).exists() {
            return None;
        }
        let folder = tempfile::tempdir().unwrap();
        let image = folder.path().join("image.dmg");
        let mount_point = folder.path().join("mounted");
        fs::create_dir(&mount_point).unwrap();
        let (file_system, name) = match format {
            Format::Apfs => ("APFS", "FresherTest"),
            Format::Fat => ("MS-DOS", "FRESHER"),
        };
        let created = hdiutil(&[
            "create".as_ref(),
            "-quiet".as_ref(),
            "-size".as_ref(),
            "32m".as_ref(),
            "-fs".as_ref(),
            file_system.as_ref(),
            "-volname".as_ref(),
            name.as_ref(),
            image.as_os_str(),
        ]);
        assert!(created, "hdiutil could not create the image");
        let attached = hdiutil(&[
            "attach".as_ref(),
            "-quiet".as_ref(),
            "-nobrowse".as_ref(),
            "-noautoopen".as_ref(),
            "-mountpoint".as_ref(),
            mount_point.as_os_str(),
            image.as_os_str(),
        ]);
        assert!(attached, "hdiutil could not attach the image");
        Some(Self { mount_point: fs::canonicalize(&mount_point).unwrap(), _folder: folder })
    }

    pub(crate) fn mount_point(&self) -> &Path {
        &self.mount_point
    }
}

impl Drop for DiskImage {
    fn drop(&mut self) {
        let point = self.mount_point.as_os_str();
        if !hdiutil(&["detach".as_ref(), "-quiet".as_ref(), point]) {
            hdiutil(&["detach".as_ref(), "-quiet".as_ref(), "-force".as_ref(), point]);
        }
    }
}

fn hdiutil(arguments: &[&OsStr]) -> bool {
    Command::new(HDIUTIL).args(arguments).status().is_ok_and(|status| status.success())
}
