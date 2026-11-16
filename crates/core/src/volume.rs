//! The volume facts the scan needs, and the space a cleanup measures.

use std::path::Path;

use crate::{ByteSize, ReadError};

/// Tells which volume a path is on.
pub trait Volumes: Send + Sync {
    fn identify(&self, path: &Path) -> Result<VolumeFacts, ReadError>;
    /// The bytes the user can still write on the volume that holds `path`: measured before and after a cleanup.
    fn available(&self, path: &Path) -> Result<ByteSize, ReadError>;
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
