//! The mounted volumes, as the system has them cached: for a check that Spotlight indexes every one of them.

use std::io;
use std::path::{Path, PathBuf};

use super::{c_string, is_local, path_of};
use crate::ffi::volume;

/// The file system types that hold no apps: the devices (`/dev`) and the automounter's maps.
const HOLDS_NO_APPS: [&[u8]; 2] = [b"devfs", b"autofs"];

/// A mounted volume: where it is mounted, whether it is on this Mac, and whether the system shows it to the user.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MountedVolume {
    pub mount_point: PathBuf,
    /// On this Mac (`MNT_LOCAL`), not over the network.
    pub local: bool,
    /// Shown by the system (no `MNT_DONTBROWSE`); its own volumes are not.
    pub browsable: bool,
}

/// Every mounted volume but the devices and the automounter's maps, from the system's cached list (`MNT_NOWAIT`: no
/// volume is asked, none blocks). A mount whose mount point cannot be read is an error, never left out.
pub fn mounted_volumes() -> io::Result<Vec<MountedVolume>> {
    let mut found = Vec::new();
    for stats in volume::mounts()? {
        found.extend(mounted(&stats)?);
    }
    Ok(found)
}

/// Where the volume that holds `path` is mounted (a link in `path` is followed): on APFS, the home is on the data
/// volume (`/System/Volumes/Data`), whatever its path says.
pub fn mount_point_of(path: &Path) -> io::Result<PathBuf> {
    let stats = volume::statfs(path)?;
    let mount_point = c_string(&stats.f_mntonname).ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))?;
    Ok(path_of(&mount_point))
}

/// The volume `stats` describes; `None` for a type that holds no apps.
fn mounted(stats: &libc::statfs) -> io::Result<Option<MountedVolume>> {
    let kind = c_string(&stats.f_fstypename).unwrap_or_default();
    if HOLDS_NO_APPS.contains(&kind.to_bytes()) {
        return Ok(None);
    }
    let mount_point = c_string(&stats.f_mntonname).ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))?;
    Ok(Some(MountedVolume {
        mount_point: path_of(&mount_point),
        local: is_local(stats.f_flags),
        browsable: stats.f_flags & libc::MNT_DONTBROWSE.cast_unsigned() == 0,
    }))
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::io;
        use std::path::Path;

        use crate::disk_image::{DiskImage, Format};
        use crate::volume::mounted::{mounted, mounted_volumes};
        use crate::volume::{mount_point_of, tests::mount};

        #[test]
        fn device_and_automount_maps_are_left_out() {
            let mut devices = mount("devfs", "/dev", 0);
            devices.f_fstypename[..5].copy_from_slice(&b"devfs".map(u8::cast_signed));
            let mut automount = mount("map auto_home", "/System/Volumes/Data/home", 0);
            automount.f_fstypename[..6].copy_from_slice(&b"autofs".map(u8::cast_signed));

            assert_eq!(mounted(&devices).unwrap(), None);
            assert_eq!(mounted(&automount).unwrap(), None);
            let listed = mounted_volumes().unwrap();
            assert!(listed.iter().all(|volume| volume.mount_point != Path::new("/dev")), "{listed:?}");
        }

        #[test]
        fn a_mount_point_that_cannot_be_read_is_an_error() {
            let mut unnamed = mount("/dev/disk9s1", "", 0);
            unnamed.f_mntonname.fill(b'a'.cast_signed());

            assert_eq!(mounted(&unnamed).unwrap_err().kind(), io::ErrorKind::InvalidData);
        }

        #[test]
        fn a_disk_image_mounted_without_browsing_is_local_and_not_browsable() {
            let Some(image) = DiskImage::mount(Format::Apfs) else {
                eprintln!("skipped: hdiutil is not on this Mac");
                return;
            };

            let listed = mounted_volumes().unwrap();

            let volume = listed.iter().find(|volume| volume.mount_point == image.mount_point());
            assert_eq!(volume.map(|volume| (volume.local, volume.browsable)), Some((true, false)), "{listed:?}");
        }

        #[test]
        fn the_mount_point_of_a_missing_path_is_an_error() {
            let root = tempfile::tempdir().unwrap();

            assert_eq!(mount_point_of(&root.path().join("missing")).unwrap_err().kind(), io::ErrorKind::NotFound);
        }
    }

    mod positive {
        use std::path::Path;

        use crate::volume::mounted::{MountedVolume, mounted, mounted_volumes};
        use crate::volume::{mount_point_of, tests::mount};

        #[test]
        fn a_local_mount_shown_by_the_system_is_local_and_browsable() {
            let mut stats = mount("/dev/disk9s1", "/Volumes/Work", 0);
            stats.f_fstypename[..4].copy_from_slice(&b"apfs".map(u8::cast_signed));
            stats.f_flags = libc::MNT_LOCAL.cast_unsigned();

            assert_eq!(
                mounted(&stats).unwrap(),
                Some(MountedVolume { mount_point: "/Volumes/Work".into(), local: true, browsable: true })
            );
        }

        #[test]
        fn a_network_mount_hidden_by_the_system_is_neither() {
            let mut stats = mount("//server/share", "/Volumes/share", 0);
            stats.f_fstypename[..5].copy_from_slice(&b"smbfs".map(u8::cast_signed));
            stats.f_flags = libc::MNT_DONTBROWSE.cast_unsigned();

            assert_eq!(
                mounted(&stats).unwrap(),
                Some(MountedVolume { mount_point: "/Volumes/share".into(), local: false, browsable: false })
            );
        }

        #[test]
        fn the_root_of_this_mac_is_listed_local_and_browsable() {
            let listed = mounted_volumes().unwrap();

            let root = listed.iter().find(|volume| volume.mount_point == Path::new("/"));

            assert_eq!(root, Some(&MountedVolume { mount_point: "/".into(), local: true, browsable: true }));
        }

        #[test]
        fn the_volume_of_a_temporary_folder_is_a_listed_mount() {
            let root = tempfile::tempdir().unwrap();

            let mount_point = mount_point_of(root.path()).unwrap();

            assert!(
                mounted_volumes().unwrap().iter().any(|volume| volume.mount_point == mount_point),
                "{mount_point:?}"
            );
        }
    }
}
