//! `MacVolumes`: which volume a path is on.

use std::ffi::CString;
use std::fs;
use std::os::unix::fs::MetadataExt as _;
use std::path::Path;

use fresher_core::{ByteSize, ReadError, VolumeFacts, VolumeId, Volumes, stable_hash};

use crate::errors::read_error;
use crate::ffi::volume;

/// The volume attributes asked of a mount point: the format capabilities (case sensitivity) and the UUID.
const LIST: libc::attrlist = libc::attrlist {
    bitmapcount: libc::ATTR_BIT_MAP_COUNT,
    reserved: 0,
    commonattr: libc::ATTR_CMN_RETURNED_ATTRS,
    volattr: libc::ATTR_VOL_INFO | libc::ATTR_VOL_CAPABILITIES | libc::ATTR_VOL_UUID,
    dirattr: 0,
    fileattr: 0,
    forkattr: 0,
};
/// The length, the returned mask, the capabilities (two sets of four `u32`) and the UUID.
const BUFFER: usize = 4 + 20 + 32 + 16;
/// The top 32 bits of an id made up for a volume with no UUID ("frsh").
const DERIVED_TAG: u128 = 0x6672_7368;

/// Tells which volume a path is on, from `statfs` and the volume's attributes. Opens nothing but the path's metadata.
#[derive(Clone, Copy, Debug, Default)]
pub struct MacVolumes;

impl MacVolumes {
    pub const fn new() -> Self {
        Self
    }
}

impl Volumes for MacVolumes {
    /// A link in `path` is followed; the device is the path's own (`st_dev`), as the bulk read reports it.
    fn identify(&self, path: &Path) -> Result<VolumeFacts, ReadError> {
        let metadata = fs::metadata(path).map_err(|error| read_error(&error))?;
        let stats = volume::statfs(path).map_err(|error| read_error(&error))?;
        let mount_point = c_string(&stats.f_mntonname).ok_or(ReadError::Failed)?;
        let mut buffer = [0_u8; BUFFER];
        let attributes = match volume::attributes(&mount_point, &LIST, &mut buffer) {
            Ok(()) => VolumeAttributes::parse(&buffer),
            Err(error) => {
                tracing::debug!(%error, "the volume's attributes are unknown");
                VolumeAttributes::default()
            }
        };
        let id = attributes.uuid.unwrap_or_else(|| {
            let from = c_string(&stats.f_mntfromname).unwrap_or_default();
            let kind = c_string(&stats.f_fstypename).unwrap_or_default();
            derived_id(kind.to_bytes(), from.to_bytes())
        });
        let used = stats.f_blocks.saturating_sub(stats.f_bfree).saturating_mul(u64::from(stats.f_bsize));
        Ok(VolumeFacts {
            id: VolumeId::new(id),
            device: metadata.dev(),
            case_sensitive: attributes.case_sensitive,
            used: ByteSize::new(used),
        })
    }

    /// The blocks free to the user (`f_bavail`, not counting the ones kept for the system) times the block size; a
    /// link in `path` is followed.
    fn available(&self, path: &Path) -> Result<ByteSize, ReadError> {
        let stats = volume::statfs(path).map_err(|error| read_error(&error))?;
        Ok(ByteSize::new(stats.f_bavail.saturating_mul(u64::from(stats.f_bsize))))
    }
}

/// Whether `id` was made up for a volume with no UUID: such a volume keeps no journal position, so it is always
/// scanned in full.
pub(crate) const fn is_derived(id: VolumeId) -> bool {
    id.get() >> 96 == DERIVED_TAG
}

/// An id for a volume with no UUID (some foreign file systems): its file system type and what is mounted.
fn derived_id(kind: &[u8], from: &[u8]) -> u128 {
    let low = u128::from(stable_hash(from));
    let kind = u128::from(stable_hash(kind) & u64::from(u32::MAX));
    DERIVED_TAG << 96 | kind << 64 | low
}

/// The NUL-terminated string at the start of a fixed `c_char` array of `statfs`.
pub(crate) fn c_string(chars: &[libc::c_char]) -> Option<CString> {
    let bytes: Vec<u8> = chars.iter().take_while(|&&char| char != 0).map(|char| char.cast_unsigned()).collect();
    (bytes.len() < chars.len()).then(|| CString::new(bytes).ok()).flatten()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct VolumeAttributes {
    uuid: Option<u128>,
    case_sensitive: bool,
}

impl VolumeAttributes {
    /// The attributes in `buffer`, in bit order (capabilities, then UUID), each only when the returned mask holds it;
    /// a UUID of zeros is no UUID.
    fn parse(buffer: &[u8]) -> Self {
        let field = |at: usize| buffer.get(at..at + 4).and_then(|bytes| bytes.try_into().ok()).map(u32::from_ne_bytes);
        let returned = field(8).unwrap_or(0);
        let mut at = 4 + 20;
        let mut attributes = Self::default();
        if returned & libc::ATTR_VOL_CAPABILITIES != 0 {
            let format = field(at).unwrap_or(0);
            let valid = field(at + 16).unwrap_or(0);
            attributes.case_sensitive = valid & format & libc::VOL_CAP_FMT_CASE_SENSITIVE != 0;
            at += 32;
        }
        if returned & libc::ATTR_VOL_UUID != 0 {
            attributes.uuid = buffer
                .get(at..at + 16)
                .and_then(|bytes| bytes.try_into().ok())
                .map(u128::from_be_bytes)
                .filter(|&uuid| uuid != 0);
        }
        attributes
    }
}

#[cfg(test)]
mod tests {
    mod negative {
        use std::path::Path;

        use fresher_core::{ReadError, VolumeId, Volumes};

        use crate::MacVolumes;
        use crate::volume::{VolumeAttributes, c_string, is_derived};

        #[test]
        fn a_path_that_does_not_exist_is_an_error() {
            let root = tempfile::tempdir().unwrap();

            assert_eq!(MacVolumes::new().identify(&root.path().join("missing")), Err(ReadError::Vanished));
            assert_eq!(MacVolumes::new().available(&root.path().join("missing")), Err(ReadError::Vanished));
            assert_eq!(MacVolumes::new().identify(Path::new("bad\0path")), Err(ReadError::Failed));
        }

        #[test]
        fn a_short_or_empty_attribute_buffer_gives_no_uuid() {
            assert_eq!(VolumeAttributes::parse(&[]), VolumeAttributes::default());
            let mut buffer = [0_u8; 40];
            buffer[8..12].copy_from_slice(&(libc::ATTR_VOL_CAPABILITIES | libc::ATTR_VOL_UUID).to_ne_bytes());

            assert_eq!(VolumeAttributes::parse(&buffer).uuid, None);
        }

        #[test]
        fn a_uuid_of_zeros_is_no_uuid() {
            let mut buffer = [0_u8; 72];
            buffer[8..12].copy_from_slice(&libc::ATTR_VOL_UUID.to_ne_bytes());

            assert_eq!(VolumeAttributes::parse(&buffer).uuid, None);
        }

        #[test]
        fn a_real_uuid_is_not_taken_for_a_derived_id() {
            assert!(!is_derived(VolumeId::new(0x1234_5678_9abc_def0_1234_5678_9abc_def0)));
        }

        #[test]
        fn a_name_without_its_nul_is_not_a_string() {
            assert_eq!(c_string(&[0x61, 0x62]), None);
        }
    }

    mod positive {
        use std::fs;

        use fresher_core::{VolumeId, Volumes};

        use crate::MacVolumes;
        use crate::disk_image::{DiskImage, Format};
        use crate::volume::{VolumeAttributes, derived_id, is_derived};

        #[test]
        fn mac_volumes_answers_the_available_space() {
            let root = tempfile::tempdir().unwrap();
            let volumes = MacVolumes::new();

            let available = volumes.available(root.path()).unwrap();

            assert!(available.bytes() > 0);
            assert!(available.bytes().is_multiple_of(512), "{available}");
        }

        #[test]
        fn the_same_volume_has_the_same_id_for_two_paths() {
            let root = tempfile::tempdir().unwrap();
            fs::create_dir(root.path().join("inner")).unwrap();
            let volumes = MacVolumes::new();

            let (outer, inner) =
                (volumes.identify(root.path()).unwrap(), volumes.identify(&root.path().join("inner")).unwrap());

            assert_eq!((outer.id, outer.device, outer.case_sensitive), (inner.id, inner.device, inner.case_sensitive));
            assert!(!is_derived(outer.id));
        }

        #[test]
        fn a_temporary_folder_is_on_the_volume_of_its_parent() {
            let root = tempfile::tempdir().unwrap();
            let volumes = MacVolumes::new();

            let (folder, parent) =
                (volumes.identify(root.path()).unwrap(), volumes.identify(root.path().parent().unwrap()).unwrap());

            assert_eq!((folder.id, folder.device), (parent.id, parent.device));
        }

        #[test]
        fn a_disk_image_has_its_own_id_and_device() {
            let Some(image) = DiskImage::mount(Format::Apfs) else {
                return;
            };
            let root = tempfile::tempdir().unwrap();
            let volumes = MacVolumes::new();

            let (on_image, here) =
                (volumes.identify(image.mount_point()).unwrap(), volumes.identify(root.path()).unwrap());

            assert_ne!(on_image.id, here.id);
            assert_ne!(on_image.device, here.device);
            assert!(!is_derived(on_image.id));
            assert!(on_image.used.bytes() < 32 * 1024 * 1024);
        }

        #[test]
        fn used_space_is_below_the_total() {
            let root = tempfile::tempdir().unwrap();
            let stats = crate::ffi::volume::statfs(root.path()).unwrap();

            let used = MacVolumes::new().identify(root.path()).unwrap().used.bytes();

            assert!(used > 0);
            assert!(used <= stats.f_blocks * u64::from(stats.f_bsize));
        }

        #[test]
        fn a_derived_id_is_marked_and_stable() {
            let id = derived_id(b"smbfs", b"//server/share");

            assert!(is_derived(VolumeId::new(id)));
            assert_eq!(id, derived_id(b"smbfs", b"//server/share"));
            assert_ne!(id, derived_id(b"nfs", b"//server/share"));
            assert_ne!(id, derived_id(b"smbfs", b"//server/other"));
        }

        #[test]
        fn the_attributes_are_read_in_bit_order() {
            let mut buffer = [0_u8; 72];
            buffer[8..12].copy_from_slice(&(libc::ATTR_VOL_CAPABILITIES | libc::ATTR_VOL_UUID).to_ne_bytes());
            buffer[24..28].copy_from_slice(&libc::VOL_CAP_FMT_CASE_SENSITIVE.to_ne_bytes());
            buffer[40..44].copy_from_slice(&libc::VOL_CAP_FMT_CASE_SENSITIVE.to_ne_bytes());
            buffer[56..72].copy_from_slice(&7_u128.to_be_bytes());

            assert_eq!(VolumeAttributes::parse(&buffer), VolumeAttributes { uuid: Some(7), case_sensitive: true });
        }
    }
}
