//! A disk image the test creates and mounts in its own temporary folder, detached when dropped (tests only).

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use tempfile::TempDir;

const HDIUTIL: &str = "/usr/bin/hdiutil";

/// A file system `hdiutil create -fs` knows.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Format {
    Apfs,
}

/// A mounted disk image. It is mounted with `-nobrowse`, so Finder does not show it (and the system keeps no change
/// journal for it); it never touches another volume.
#[derive(Debug)]
pub(crate) struct DiskImage {
    mount_point: PathBuf,
    image: PathBuf,
    _folder: TempDir,
}

impl DiskImage {
    /// `None` when `hdiutil` is not on this Mac: the test that needs it is skipped.
    pub(crate) fn mount(format: Format) -> Option<Self> {
        Self::create(format, |_| ())
    }

    /// An image attached `-readonly` (a volume with no Trash, 01/01), after `fill` wrote its items while it was
    /// still writable. `None` when `hdiutil` is not on this Mac.
    pub(crate) fn mount_read_only(format: Format, fill: impl FnOnce(&Path)) -> Option<Self> {
        let image = Self::create(format, fill)?;
        let point = image.mount_point.as_os_str();
        assert!(hdiutil(&["detach".as_ref(), "-quiet".as_ref(), point]), "hdiutil could not detach the image");
        attach(&image.image, &image.mount_point, true);
        Some(image)
    }

    fn create(format: Format, fill: impl FnOnce(&Path)) -> Option<Self> {
        if !Path::new(HDIUTIL).exists() {
            return None;
        }
        let folder = tempfile::tempdir().unwrap();
        let image = folder.path().join("image.dmg");
        let mount_point = folder.path().join("mounted");
        fs::create_dir(&mount_point).unwrap();
        let (file_system, name) = match format {
            Format::Apfs => ("APFS", "FresherTest"),
        };
        let created = hdiutil_retrying(&[
            "create".as_ref(),
            "-quiet".as_ref(),
            "-ov".as_ref(),
            "-size".as_ref(),
            "32m".as_ref(),
            "-fs".as_ref(),
            file_system.as_ref(),
            "-volname".as_ref(),
            name.as_ref(),
            image.as_os_str(),
        ]);
        assert!(created.is_ok(), "hdiutil could not create the image: {created:?}");
        attach(&image, &mount_point, false);
        let mount_point = fs::canonicalize(&mount_point).unwrap();
        fill(&mount_point);
        Some(Self { mount_point, image, _folder: folder })
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

fn attach(image: &Path, mount_point: &Path, read_only: bool) {
    let mut arguments: Vec<&OsStr> =
        vec!["attach".as_ref(), "-quiet".as_ref(), "-nobrowse".as_ref(), "-noautoopen".as_ref()];
    if read_only {
        arguments.push("-readonly".as_ref());
    }
    arguments.extend(["-mountpoint".as_ref(), mount_point.as_os_str(), image.as_os_str()]);
    let attached = hdiutil_retrying(&arguments);
    assert!(attached.is_ok(), "hdiutil could not attach the image: {attached:?}");
}

/// `hdiutil` now and then fails when many tests make images at once: a few tries, apart, before the test fails.
fn hdiutil_retrying(arguments: &[&OsStr]) -> Result<(), String> {
    const TRIES: u32 = 4;
    let mut error = String::new();
    for attempt in 1..=TRIES {
        match Command::new(HDIUTIL).args(arguments).output() {
            Ok(output) if output.status.success() => return Ok(()),
            Ok(output) => error = String::from_utf8_lossy(&output.stderr).into_owned(),
            Err(failure) => error = failure.to_string(),
        }
        std::thread::sleep(Duration::from_millis(250) * attempt);
    }
    Err(error)
}

fn hdiutil(arguments: &[&OsStr]) -> bool {
    Command::new(HDIUTIL).args(arguments).status().is_ok_and(|status| status.success())
}
