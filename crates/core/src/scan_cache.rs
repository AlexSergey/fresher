//! The cache of a scan scope: the last committed state, so a launch shows it at once and a rescan reads only what
//! changed. Safe to lose: a missing or broken cache means a full scan.

use crate::{ByteSize, StoreError};

/// Keeps two generations of the cache body per scope, within a cap; implemented by the store.
pub trait ScanCache: Send + Sync {
    /// The largest body `save` accepts.
    fn cap(&self) -> ByteSize;
    /// The body of the current generation for `scope`; `None` when there is none or it was discarded.
    fn load(&self, scope: ScopeKey) -> Result<Option<Vec<u8>>, StoreError>;
    /// The body of the previous generation (for plan 010).
    fn load_previous(&self, scope: ScopeKey) -> Result<Option<Vec<u8>>, StoreError>;
    /// Makes `body` the current generation of `scope`; the current one becomes the previous.
    fn save(&self, scope: ScopeKey, body: &[u8]) -> Result<(), StoreError>;
}

/// What a cache is of: a stable hash of the canonical scan roots and their volume ids.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeKey(u64);

impl ScopeKey {
    pub const fn new(key: u64) -> Self {
        Self(key)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}
