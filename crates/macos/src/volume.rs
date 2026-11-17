//! `MacVolumes`: which volume a path is on, and the space it shares.

use std::ffi::{CStr, CString, OsStr};
use std::fs;
use std::os::unix::ffi::OsStrExt as _;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};

use fresher_core::{
    ByteSize, DiskSpace, MountKind, Neighbour, NeighbourRole, ReadError, VolumeFacts, VolumeId, Volumes, stable_hash,
};

use crate::errors::read_error;
use crate::ffi::volume::{self, VolumeResources};

/// The volume attributes asked of a mount point: the name, the format capabilities (case sensitivity), the UUID and the
/// space the volume itself uses.
const LIST: libc::attrlist = libc::attrlist {
    bitmapcount: libc::ATTR_BIT_MAP_COUNT,
    reserved: 0,
    commonattr: libc::ATTR_CMN_RETURNED_ATTRS,
    volattr: libc::ATTR_VOL_INFO
        | libc::ATTR_VOL_NAME
        | libc::ATTR_VOL_CAPABILITIES
        | libc::ATTR_VOL_UUID
        | libc::ATTR_VOL_SPACEUSED,
    dirattr: 0,
    fileattr: 0,
    forkattr: 0,
};
/// The length, the returned mask, the space used, the name's reference, the capabilities (two sets of four `u32`), the
/// UUID, and room for the name itself (at most 255 characters of up to four bytes each, and its NUL).
const BUFFER: usize = 4 + 20 + 8 + 32 + 16 + 8 + 1024;
/// The device names of the slices of a disk: `/dev/disk<N>s<M>`, and `s<K>` again for a snapshot of a slice.
const DISK_PREFIX: &[u8] = b"/dev/disk";
/// Where the system mounts its own volumes beside the user's data.
const SYSTEM_VOLUMES: &str = "/System/Volumes";
/// The system's volume of swap files and the sleep image.
const VIRTUAL_MEMORY: &str = "/System/Volumes/VM";
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
        let attributes = VolumeAttributes::read(&mount_point);
        let id = attributes.uuid.unwrap_or_else(|| {
            let from = c_string(&stats.f_mntfromname).unwrap_or_default();
            let kind = c_string(&stats.f_fstypename).unwrap_or_default();
            derived_id(kind.to_bytes(), from.to_bytes())
        });
        Ok(VolumeFacts {
            id: VolumeId::new(id),
            device: metadata.dev(),
            case_sensitive: attributes.case_sensitive,
            used: used(&stats, &attributes),
        })
    }

    /// The blocks free to the user (`f_bavail`, not counting the ones kept for the system) times the block size; a
    /// link in `path` is followed.
    fn available(&self, path: &Path) -> Result<ByteSize, ReadError> {
        let stats = volume::statfs(path).map_err(|error| read_error(&error))?;
        Ok(ByteSize::new(stats.f_bavail.saturating_mul(u64::from(stats.f_bsize))))
    }

    /// The capacity is the whole container's (`f_blocks`); the neighbours come from the system's cached list of
    /// mounts, and only they, on the same local disk, are asked their name and used space. A link in `path` is
    /// followed.
    fn space(&self, path: &Path) -> Result<DiskSpace, ReadError> {
        let stats = volume::statfs(path).map_err(|error| read_error(&error))?;
        let mount_point = c_string(&stats.f_mntonname).ok_or(ReadError::Failed)?;
        let attributes = VolumeAttributes::read(&mount_point);
        let mount_point = path_of(&mount_point);
        let free = stats.f_bavail.saturating_mul(u64::from(stats.f_bsize));
        let unknown = VolumeResources { important_available: None, internal: None };
        // A network volume is not asked: it is never internal, and purgeable space is a local volume's.
        let resources = if is_local(stats.f_flags) {
            volume::resources(path).unwrap_or_else(|error| {
                tracing::debug!(%error, "the volume's resources are unknown");
                unknown
            })
        } else {
            unknown
        };
        let used = used(&stats, &attributes);
        Ok(DiskSpace {
            name: name_of(&mount_point, attributes.name),
            kind: kind_of(stats.f_flags, resources.internal),
            capacity: ByteSize::new(stats.f_blocks.saturating_mul(u64::from(stats.f_bsize))),
            used,
            free: ByteSize::new(free),
            purgeable: purgeable_of(resources.important_available, free),
            neighbours: neighbours(&stats),
            mount_point,
        })
    }
}

/// The bytes the volume itself holds: the space used its attributes give, else its blocks less the free ones (those
/// kept for the system included). On APFS the blocks are the container's, shared by all its volumes, so only the
/// attribute tells one volume's share.
fn used(stats: &libc::statfs, attributes: &VolumeAttributes) -> ByteSize {
    ByteSize::new(
        attributes
            .space_used
            .unwrap_or_else(|| stats.f_blocks.saturating_sub(stats.f_bfree).saturating_mul(u64::from(stats.f_bsize))),
    )
}

/// Whether a mount of `flags` is on this Mac rather than over the network.
const fn is_local(flags: u32) -> bool {
    flags & libc::MNT_LOCAL.cast_unsigned() != 0
}

/// `Network` when the mount is not local, whatever else is known; `Internal` only when the system says so: a volume
/// it gives no answer for (a mounted disk image) is `External`.
const fn kind_of(flags: u32, internal: Option<bool>) -> MountKind {
    if !is_local(flags) {
        MountKind::Network
    } else if matches!(internal, Some(true)) {
        MountKind::Internal
    } else {
        MountKind::External
    }
}

/// What the system can purge: the capacity for important usage beyond the free space, never below zero.
fn purgeable_of(important_available: Option<u64>, free: u64) -> Option<ByteSize> {
    important_available.map(|important| ByteSize::new(important.saturating_sub(free)))
}

/// The volume's own name, else the last name of its mount point, else the mount point itself.
fn name_of(mount_point: &Path, read: Option<String>) -> String {
    read.or_else(|| mount_point.file_name().map(|name| name.to_string_lossy().into_owned()))
        .unwrap_or_else(|| mount_point.to_string_lossy().into_owned())
}

fn path_of(chars: &CStr) -> PathBuf {
    PathBuf::from(OsStr::from_bytes(chars.to_bytes()))
}

/// The disk number of a slice's device name (`/dev/disk3s1s1` and `/dev/disk3s5` are of disk 3); `None` for a name
/// that is not a slice of a disk (a whole disk, a network share, a RAM disk, `devfs`).
fn disk_of(from: &[u8]) -> Option<u32> {
    let rest = from.strip_prefix(DISK_PREFIX)?;
    let digits = rest.iter().take_while(|byte| byte.is_ascii_digit()).count();
    let (number, slice) = rest.split_at(digits);
    let slice = slice.strip_prefix(b"s")?;
    if number.is_empty() || !slice.first().is_some_and(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(number).ok()?.parse().ok()
}

/// What a volume that shares the disk is for, told by where it is mounted.
fn role_of(mount_point: &Path) -> NeighbourRole {
    if mount_point == Path::new(VIRTUAL_MEMORY) {
        NeighbourRole::VirtualMemory
    } else if mount_point == Path::new("/") || mount_point.starts_with(SYSTEM_VOLUMES) {
        NeighbourRole::System
    } else {
        NeighbourRole::Other
    }
}

/// Whether `other` is another volume of the container of `this`: a slice of the same disk, of the same size.
fn shares_the_disk(this: &libc::statfs, other: &libc::statfs) -> bool {
    let disk = |stats: &libc::statfs| c_string(&stats.f_mntfromname).and_then(|from| disk_of(from.to_bytes()));
    let mount_point = |stats: &libc::statfs| c_string(&stats.f_mntonname);
    disk(this).is_some_and(|disk_number| disk(other) == Some(disk_number))
        && this.f_blocks == other.f_blocks
        && mount_point(this) != mount_point(other)
}

/// The other volumes of the container of `stats`, from the system's cached list of mounts; none when it is unknown.
fn neighbours(stats: &libc::statfs) -> Vec<Neighbour> {
    let mounts = volume::mounts().unwrap_or_else(|error| {
        tracing::debug!(%error, "the mounted volumes are unknown");
        Vec::new()
    });
    mounts
        .iter()
        .filter(|other| shares_the_disk(stats, other))
        .filter_map(|other| {
            let chars = c_string(&other.f_mntonname)?;
            let mount_point = path_of(&chars);
            let attributes = VolumeAttributes::read(&chars);
            let used = used(other, &attributes);
            Some(Neighbour { name: name_of(&mount_point, attributes.name), role: role_of(&mount_point), used })
        })
        .collect()
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct VolumeAttributes {
    name: Option<String>,
    uuid: Option<u128>,
    case_sensitive: bool,
    space_used: Option<u64>,
}

impl VolumeAttributes {
    /// The attributes of the volume mounted at `mount_point`; none known when the call fails.
    fn read(mount_point: &CStr) -> Self {
        let mut buffer = [0_u8; BUFFER];
        match volume::attributes(mount_point, &LIST, &mut buffer) {
            Ok(()) => Self::parse(&buffer),
            Err(error) => {
                tracing::debug!(%error, "the volume's attributes are unknown");
                Self::default()
            }
        }
    }

    /// The attributes in `buffer`, in the order the system writes them (the space used, the name, the capabilities,
    /// then the UUID: not the bits' order), each only when the returned mask holds it; an empty name is no name and a
    /// UUID of zeros is no UUID.
    fn parse(buffer: &[u8]) -> Self {
        let field = |at: usize| buffer.get(at..at + 4).and_then(|bytes| bytes.try_into().ok()).map(u32::from_ne_bytes);
        let returned = field(8).unwrap_or(0);
        let mut at = 4 + 20;
        let mut attributes = Self::default();
        if returned & libc::ATTR_VOL_SPACEUSED != 0 {
            attributes.space_used =
                buffer.get(at..at + 8).and_then(|bytes| bytes.try_into().ok()).map(u64::from_ne_bytes);
            at += 8;
        }
        if returned & libc::ATTR_VOL_NAME != 0 {
            attributes.name = Self::name(buffer, at, field(at), field(at + 4));
            at += 8;
        }
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

    /// The name an `attrreference_t` at `at` points to: `offset` bytes from the reference, `length` bytes with its NUL.
    fn name(buffer: &[u8], at: usize, offset: Option<u32>, length: Option<u32>) -> Option<String> {
        let start = at.checked_add_signed(isize::try_from(offset?.cast_signed()).ok()?)?;
        let end = start.checked_add(usize::try_from(length?).ok()?)?;
        let bytes = buffer.get(start..end)?;
        let bytes = bytes.split(|&byte| byte == 0).next().unwrap_or_default();
        let name = std::str::from_utf8(bytes).ok()?;
        (!name.is_empty()).then(|| name.to_owned())
    }
}

#[cfg(test)]
mod tests {
    /// A mount as `getfsstat` lists it: what is mounted, where, and the container's size in blocks.
    pub(super) fn mount(from: &str, on: &str, blocks: u64) -> libc::statfs {
        let mut stats = libc::statfs { f_blocks: blocks, f_bsize: 4096, ..libc::statfs::default() };
        for (field, text) in [(&mut stats.f_mntfromname, from), (&mut stats.f_mntonname, on)] {
            for (char, byte) in field.iter_mut().zip(text.bytes()) {
                *char = byte.cast_signed();
            }
        }
        stats
    }

    mod negative {
        use std::path::Path;

        use fresher_core::{ByteSize, MountKind, ReadError, VolumeId, Volumes};

        use crate::MacVolumes;
        use crate::volume::tests::mount;
        use crate::volume::{VolumeAttributes, c_string, disk_of, is_derived, kind_of, purgeable_of, shares_the_disk};

        #[test]
        fn a_path_that_does_not_exist_is_an_error() {
            let root = tempfile::tempdir().unwrap();

            assert_eq!(MacVolumes::new().identify(&root.path().join("missing")), Err(ReadError::Vanished));
            assert_eq!(MacVolumes::new().available(&root.path().join("missing")), Err(ReadError::Vanished));
            assert_eq!(MacVolumes::new().identify(Path::new("bad\0path")), Err(ReadError::Failed));
            assert_eq!(MacVolumes::new().space(&root.path().join("missing")), Err(ReadError::Vanished));
        }

        #[test]
        fn disk_of_refuses_a_name_that_is_not_a_disk() {
            for from in ["map auto_home", "devfs", "//server/share", "/dev/rdisk3", "/dev/disk3", "/dev/disks1", ""] {
                assert_eq!(disk_of(from.as_bytes()), None, "{from}");
            }
        }

        #[test]
        fn a_volume_on_another_disk_is_not_a_neighbour() {
            let data = mount("/dev/disk3s5", "/System/Volumes/Data", 1000);

            assert!(!shares_the_disk(&data, &mount("/dev/disk4s1", "/Volumes/Backup", 1000)));
            assert!(!shares_the_disk(&data, &mount("/dev/disk3s5", "/System/Volumes/Data", 1000)));
            assert!(!shares_the_disk(&data, &mount("devfs", "/dev", 1000)));
            assert!(!shares_the_disk(&mount("map auto_home", "/home", 1000), &mount("map -hosts", "/net", 1000)));
        }

        #[test]
        fn a_network_mount_is_network_whatever_internal_says() {
            assert_eq!(kind_of(0, Some(true)), MountKind::Network);
            assert_eq!(kind_of(0, None), MountKind::Network);
        }

        #[test]
        fn a_volume_the_system_gives_no_answer_for_is_not_internal() {
            let local = libc::MNT_LOCAL.cast_unsigned();

            assert_eq!(kind_of(local, None), MountKind::External);
            assert_eq!(kind_of(local, Some(false)), MountKind::External);
        }

        #[test]
        fn purgeable_never_goes_below_zero() {
            assert_eq!(purgeable_of(Some(10), 20), Some(ByteSize::new(0)));
            assert_eq!(purgeable_of(None, 20), None);
        }

        #[test]
        fn a_name_reference_out_of_the_buffer_gives_no_name() {
            let mut buffer = [0_u8; 40];
            buffer[8..12].copy_from_slice(&libc::ATTR_VOL_NAME.to_ne_bytes());
            buffer[24..28].copy_from_slice(&100_u32.to_ne_bytes());
            buffer[28..32].copy_from_slice(&4_u32.to_ne_bytes());

            assert_eq!(VolumeAttributes::parse(&buffer).name, None);
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
        use std::path::Path;

        use fresher_core::{ByteSize, MountKind, NeighbourRole, VolumeId, Volumes};

        use crate::MacVolumes;
        use crate::disk_image::{DiskImage, Format};
        use crate::volume::tests::mount;
        use crate::volume::{
            VolumeAttributes, derived_id, disk_of, is_derived, kind_of, name_of, purgeable_of, role_of, shares_the_disk,
        };

        const MIB: u64 = 1024 * 1024;

        #[test]
        fn disk_of_reads_the_disk_number() {
            assert_eq!(disk_of(b"/dev/disk3s1s1"), Some(3));
            assert_eq!(disk_of(b"/dev/disk3s5"), Some(3));
            assert_eq!(disk_of(b"/dev/disk12s2"), Some(12));
        }

        #[test]
        fn roles_follow_the_mount_points() {
            assert_eq!(role_of(Path::new("/")), NeighbourRole::System);
            assert_eq!(role_of(Path::new("/System/Volumes/VM")), NeighbourRole::VirtualMemory);
            assert_eq!(role_of(Path::new("/System/Volumes/Preboot")), NeighbourRole::System);
            assert_eq!(role_of(Path::new("/System/Volumes/Update")), NeighbourRole::System);
            assert_eq!(role_of(Path::new("/Volumes/Work")), NeighbourRole::Other);
            assert_eq!(role_of(Path::new("/System/VolumesExtra")), NeighbourRole::Other);
        }

        #[test]
        fn a_slice_of_the_same_container_is_a_neighbour() {
            let data = mount("/dev/disk3s5", "/System/Volumes/Data", 1000);

            assert!(shares_the_disk(&data, &mount("/dev/disk3s1s1", "/", 1000)));
            assert!(shares_the_disk(&data, &mount("/dev/disk3s6", "/System/Volumes/VM", 1000)));
            assert!(!shares_the_disk(&data, &mount("/dev/disk3s7", "/Volumes/Other", 2000)));
        }

        #[test]
        fn the_kind_and_purgeable_space_follow_what_the_system_says() {
            assert_eq!(kind_of(libc::MNT_LOCAL.cast_unsigned(), Some(true)), MountKind::Internal);
            assert_eq!(purgeable_of(Some(30), 20), Some(ByteSize::new(10)));
        }

        #[test]
        fn a_volume_is_named_by_itself_else_by_its_mount_point() {
            assert_eq!(name_of(Path::new("/Volumes/Work"), Some("Work Disk".to_owned())), "Work Disk");
            assert_eq!(name_of(Path::new("/Volumes/Work"), None), "Work");
            assert_eq!(name_of(Path::new("/"), None), "/");
        }

        #[test]
        fn the_space_of_a_temporary_folder_adds_up() {
            let root = tempfile::tempdir().unwrap();
            let volumes = MacVolumes::new();

            let before = volumes.available(root.path()).unwrap().bytes();
            let space = volumes.space(root.path()).unwrap();
            let after = volumes.available(root.path()).unwrap().bytes();

            assert!(space.capacity.bytes() >= space.used.bytes() + space.free.bytes(), "{space:?}");
            // Other programs write while the test runs: free lies between the two readings, give or take 1 MiB.
            let (low, high) = (before.min(after).saturating_sub(MIB), before.max(after) + MIB);
            assert!((low..=high).contains(&space.free.bytes()), "{space:?} against {before} and {after}");
            assert_ne!(space.name, "");
            assert_ne!(space.kind, MountKind::Network);
        }

        #[test]
        fn a_disk_image_has_no_neighbours_and_is_external() {
            let Some(image) = DiskImage::mount(Format::Apfs) else {
                eprintln!("skipped: hdiutil is not on this Mac");
                return;
            };

            let space = MacVolumes::new().space(image.mount_point()).unwrap();

            assert_eq!(space.kind, MountKind::External);
            assert_eq!(space.neighbours, Vec::new());
            assert_eq!(space.name, "FresherTest");
            assert_eq!(space.mount_point, image.mount_point());
            assert!(space.capacity.bytes() <= 32 * MIB);
        }

        #[test]
        fn the_data_volume_sees_the_system_as_a_neighbour() {
            let data = Path::new("/System/Volumes/Data");
            if !data.exists() {
                eprintln!("skipped: {} does not exist on this Mac", data.display());
                return;
            }

            let space = MacVolumes::new().space(data).unwrap();

            let roles: Vec<NeighbourRole> = space.neighbours.iter().map(|neighbour| neighbour.role).collect();
            assert!(roles.contains(&NeighbourRole::System), "{space:?}");
            assert!(space.neighbours.iter().all(|neighbour| !neighbour.name.is_empty()), "{space:?}");
            // On APFS each volume reports its own share: together they fit in the container they share.
            let neighbours: u64 = space.neighbours.iter().map(|neighbour| neighbour.used.bytes()).sum();
            assert!(space.used.bytes() + neighbours + space.free.bytes() <= space.capacity.bytes(), "{space:?}");
        }

        #[test]
        fn the_name_is_read_before_the_capabilities() {
            let mut buffer = [0_u8; 96];
            buffer[8..12].copy_from_slice(&(libc::ATTR_VOL_NAME | libc::ATTR_VOL_UUID).to_ne_bytes());
            buffer[24..28].copy_from_slice(&24_u32.to_ne_bytes());
            buffer[28..32].copy_from_slice(&5_u32.to_ne_bytes());
            buffer[32..48].copy_from_slice(&9_u128.to_be_bytes());
            buffer[48..53].copy_from_slice(b"Work\0");

            let attributes = VolumeAttributes::parse(&buffer);

            assert_eq!((attributes.name.as_deref(), attributes.uuid), (Some("Work"), Some(9)));
        }

        #[test]
        fn the_space_used_is_read_before_the_name() {
            let mut buffer = [0_u8; 72];
            buffer[8..12].copy_from_slice(&(libc::ATTR_VOL_SPACEUSED | libc::ATTR_VOL_UUID).to_ne_bytes());
            buffer[24..32].copy_from_slice(&4096_u64.to_ne_bytes());
            buffer[32..48].copy_from_slice(&9_u128.to_be_bytes());

            let attributes = VolumeAttributes::parse(&buffer);

            assert_eq!((attributes.uuid, attributes.space_used), (Some(9), Some(4096)));
        }

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
        fn the_capabilities_are_read_before_the_uuid() {
            let mut buffer = [0_u8; 72];
            buffer[8..12].copy_from_slice(&(libc::ATTR_VOL_CAPABILITIES | libc::ATTR_VOL_UUID).to_ne_bytes());
            buffer[24..28].copy_from_slice(&libc::VOL_CAP_FMT_CASE_SENSITIVE.to_ne_bytes());
            buffer[40..44].copy_from_slice(&libc::VOL_CAP_FMT_CASE_SENSITIVE.to_ne_bytes());
            buffer[56..72].copy_from_slice(&7_u128.to_be_bytes());

            assert_eq!(
                VolumeAttributes::parse(&buffer),
                VolumeAttributes { name: None, uuid: Some(7), case_sensitive: true, space_used: None }
            );
        }
    }
}
