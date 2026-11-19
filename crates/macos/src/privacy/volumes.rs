//! What guards a scan root by the volume it is on.

use std::path::Path;

use fresher_core::MountKind;

use super::places::Guard;
use crate::ffi::volume;
use crate::volume::{is_local, kind_of};

/// The guard of the volume `root` is on: a removable volume or a network one; `None` on an internal volume, and when
/// the root's `statfs` refuses (its walk then fails to read it and counts it). It asks `statfs`, then, on a local
/// volume only, the volume's resources (whether it is internal); a volume with no answer is removable, as
/// `MacVolumes::space` says. Neither query raises a prompt: from documentation ("Probe results"), unverified until
/// the user's run of plan 006/01.
pub fn volume_guard(root: &Path) -> Option<Guard> {
    let stats = volume::statfs(root).ok()?;
    // A network volume is not asked: it is never internal.
    let internal = if is_local(stats.f_flags) {
        volume::resources(root).ok().and_then(|resources| resources.internal)
    } else {
        None
    };
    guard_of(kind_of(stats.f_flags, internal))
}

const fn guard_of(kind: MountKind) -> Option<Guard> {
    match kind {
        MountKind::Internal => None,
        MountKind::External => Some(Guard::RemovableVolume),
        MountKind::Network => Some(Guard::NetworkVolume),
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::path::Path;

        use fresher_core::MountKind;

        use super::super::{guard_of, volume_guard};

        #[test]
        fn an_internal_volume_has_no_guard() {
            assert_eq!(guard_of(MountKind::Internal), None);
        }

        #[test]
        fn a_root_that_does_not_exist_has_no_guard() {
            assert_eq!(volume_guard(Path::new("/nowhere/root/that/does/not/exist")), None);
        }

        #[test]
        fn a_temporary_folder_on_the_internal_disk_has_no_guard() {
            let root = tempfile::tempdir().unwrap();

            assert_eq!(volume_guard(root.path()), None);
        }
    }

    mod positive {
        use fresher_core::MountKind;

        use super::super::{guard_of, volume_guard};
        use crate::Guard;
        use crate::disk_image::{DiskImage, Format};

        #[test]
        fn external_and_network_volumes_have_their_guard() {
            assert_eq!(guard_of(MountKind::External), Some(Guard::RemovableVolume));
            assert_eq!(guard_of(MountKind::Network), Some(Guard::NetworkVolume));
        }

        #[test]
        fn a_mounted_disk_image_is_a_removable_volume() {
            let Some(image) = DiskImage::mount(Format::Apfs) else {
                eprintln!("skipped: hdiutil is not on this Mac");
                return;
            };

            assert_eq!(volume_guard(image.mount_point()), Some(Guard::RemovableVolume));
        }
    }
}
