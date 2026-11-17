//! The volume facts the scan needs, and the space a cleanup measures.

use std::path::{Path, PathBuf};

use serde::Serialize;
use ts_rs::TS;

use crate::{ByteSize, ReadError, TextKey};

/// Tells which volume a path is on.
pub trait Volumes: Send + Sync {
    fn identify(&self, path: &Path) -> Result<VolumeFacts, ReadError>;
    /// The bytes the user can still write on the volume that holds `path`: measured before and after a cleanup.
    fn available(&self, path: &Path) -> Result<ByteSize, ReadError>;
    /// The space of the volume that holds `path`; a link in `path` is followed, as `identify`.
    fn space(&self, path: &Path) -> Result<DiskSpace, ReadError>;
}

/// A volume's identity, stable across reboots (the volume's UUID on macOS).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VolumeId(u128);

impl VolumeId {
    pub const fn new(id: u128) -> Self {
        Self(id)
    }

    pub const fn get(self) -> u128 {
        self.0
    }
}

/// What the scan knows of a volume.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VolumeFacts {
    pub id: VolumeId,
    /// The device number of the mount: it changes between mounts, the id does not.
    pub device: u64,
    /// Whether names that differ only in case are different names.
    pub case_sensitive: bool,
    /// The bytes in use on the volume.
    pub used: ByteSize,
}

/// How a volume is attached: it names the map ("Macintosh HD · internal") and says when purgeable space is not asked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
pub enum MountKind {
    /// Inside the Mac.
    Internal,
    /// Plugged in, or a mounted disk image.
    External,
    /// Over the network.
    Network,
}

impl MountKind {
    /// The kind's word for the map's name: `core.storage.kind.<kind>`.
    pub fn text_key(self) -> TextKey {
        let kind = match self {
            Self::Internal => "internal",
            Self::External => "external",
            Self::Network => "network",
        };
        TextKey::from_valid(format!("core.storage.kind.{kind}"))
    }
}

/// The space of a volume, and of the volumes that share it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiskSpace {
    pub name: String,
    pub mount_point: PathBuf,
    pub kind: MountKind,
    /// The space the volume shares: on APFS, the container's.
    pub capacity: ByteSize,
    /// What this volume itself holds.
    pub used: ByteSize,
    /// What the user can still write.
    pub free: ByteSize,
    /// What the system can free on its own; `None` when it does not say.
    pub purgeable: Option<ByteSize>,
    /// The other volumes that share the capacity.
    pub neighbours: Vec<Neighbour>,
}

/// A volume that shares the capacity with the one asked about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Neighbour {
    pub name: String,
    pub role: NeighbourRole,
    pub used: ByteSize,
}

/// What a neighbouring volume is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeighbourRole {
    System,
    VirtualMemory,
    Other,
}
