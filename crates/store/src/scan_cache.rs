//! The scan cache: the core's `ScanCache` on fixed slot files in the data folder.
//!
//! Each slot is two files, `scan-<n>.bin` (the current generation) and `scan-<n>.previous.bin`. A save rotates by
//! renaming and writes atomically; no file is ever deleted, so the cache never takes more than its slots.

use std::fs::{self, File};
use std::io::{self, Read};
use std::num::NonZeroU8;
use std::time::SystemTime;

use fresher_core::{ByteSize, ScanCache, ScopeKey, StoreError};

use crate::DataFolder;
use crate::atomic::{commit, file_path, io_error, read_file, stage};
use crate::envelope::{self, HEADER_LEN, Opened};

/// How much the scan cache keeps. The format does not depend on it: plan 010 changes the policy, not the files.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CachePolicy {
    /// How many scan scopes are kept, one slot each; every slot holds two generations.
    pub scopes: NonZeroU8,
    /// The largest file a slot keeps on disk: the body after compression, with its envelope.
    pub cap: ByteSize,
    /// The largest body, before compression, that is saved or read back; it bounds the memory a load takes.
    pub body_limit: ByteSize,
}

impl CachePolicy {
    /// Two scopes, 8 MiB a file (32 MiB in all), bodies up to 64 MiB before compression.
    pub const DEFAULT: Self = Self {
        scopes: NonZeroU8::MIN.saturating_add(1),
        cap: ByteSize::new(8 * 1024 * 1024),
        body_limit: ByteSize::new(64 * 1024 * 1024),
    };
}

/// The scan cache in the data folder.
#[derive(Debug)]
pub struct FileScanCache {
    folder: DataFolder,
    policy: CachePolicy,
}

/// One scope's place: its current file and its previous one.
#[derive(Clone, Debug)]
struct Slot {
    current: String,
    previous: String,
}

impl FileScanCache {
    /// The scan cache in `folder` with the default policy.
    pub fn new(folder: DataFolder) -> Self {
        Self::with_policy(folder, CachePolicy::DEFAULT)
    }

    pub fn with_policy(folder: DataFolder, policy: CachePolicy) -> Self {
        Self { folder, policy }
    }

    fn slots(&self) -> impl Iterator<Item = Slot> {
        (1..=self.policy.scopes.get())
            .map(|number| Slot { current: current_file_name(number), previous: format!("scan-{number}.previous.bin") })
    }

    /// The slot whose current file is of `scope`.
    fn slot_of(&self, scope: ScopeKey) -> Result<Option<Slot>, StoreError> {
        for slot in self.slots() {
            if self.scope_in(&slot.current)? == Some(scope) {
                return Ok(Some(slot));
            }
        }
        Ok(None)
    }

    /// The slot a new scope takes: a free one, else the one whose current file was written longest ago.
    fn slot_to_take(&self) -> Result<Slot, StoreError> {
        let mut oldest: Option<(SystemTime, Slot)> = None;
        for slot in self.slots() {
            let Some(modified) = self.modified(&slot.current)? else {
                return Ok(slot);
            };
            if oldest.as_ref().is_none_or(|(time, _)| modified < *time) {
                oldest = Some((modified, slot));
            }
        }
        // `scopes` is never zero, so a slot was seen.
        oldest.map(|(_, slot)| slot).ok_or(StoreError::Io { kind: io::ErrorKind::NotFound })
    }

    /// The scope of the file `name`, from its header alone; `None` when there is no such file or it is not a cache
    /// file of this version.
    fn scope_in(&self, name: &str) -> Result<Option<ScopeKey>, StoreError> {
        let file = match File::open(file_path(&self.folder, name)?) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(io_error(&error)),
        };
        let mut header = Vec::with_capacity(HEADER_LEN);
        file.take(HEADER_LEN as u64).read_to_end(&mut header).map_err(|error| io_error(&error))?;
        Ok(envelope::scope_of(&header))
    }

    fn modified(&self, name: &str) -> Result<Option<SystemTime>, StoreError> {
        match fs::metadata(file_path(&self.folder, name)?) {
            Ok(metadata) => metadata.modified().map(Some).map_err(|error| io_error(&error)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(io_error(&error)),
        }
    }

    /// The body in the file `name` if it is a whole cache file of `scope`; a discarded file is logged and is `None`.
    fn read(&self, name: &str, scope: ScopeKey) -> Result<Option<Vec<u8>>, StoreError> {
        let Some(bytes) = read_file(&self.folder, name)? else {
            return Ok(None);
        };
        let limit = usize::try_from(self.policy.body_limit.bytes()).unwrap_or(usize::MAX);
        match envelope::open(&bytes, scope, limit) {
            Opened::Body(body) => Ok(Some(body)),
            Opened::Discarded(reason) => {
                tracing::warn!(?reason, "a scan cache file was discarded");
                Ok(None)
            }
        }
    }
}

impl ScanCache for FileScanCache {
    fn cap(&self) -> ByteSize {
        self.policy.cap
    }

    fn load(&self, scope: ScopeKey) -> Result<Option<Vec<u8>>, StoreError> {
        match self.slot_of(scope)? {
            Some(slot) => self.read(&slot.current, scope),
            None => Ok(None),
        }
    }

    fn load_previous(&self, scope: ScopeKey) -> Result<Option<Vec<u8>>, StoreError> {
        // A slot taken over from another scope keeps that scope's previous file until its first rotation: the
        // envelope's scope check answers `None` for it.
        match self.slot_of(scope)? {
            Some(slot) => self.read(&slot.previous, scope),
            None => Ok(None),
        }
    }

    /// Refuses a body over the body limit or a file over the cap. The new file is staged before anything moves, so a
    /// failed write leaves both generations as they were.
    fn save(&self, scope: ScopeKey, body: &[u8]) -> Result<(), StoreError> {
        let body_size = size_of(body);
        if body_size > self.policy.body_limit {
            return Err(StoreError::TooLarge { size: body_size, cap: self.policy.body_limit });
        }
        let sealed = envelope::seal(scope, body);
        let sealed_size = size_of(&sealed);
        if sealed_size > self.policy.cap {
            return Err(StoreError::TooLarge { size: sealed_size, cap: self.policy.cap });
        }
        let (slot, rotate) = match self.slot_of(scope)? {
            Some(slot) => (slot, true),
            None => (self.slot_to_take()?, false),
        };
        let staged = stage(&self.folder, &slot.current, &sealed)?;
        if rotate {
            let current = file_path(&self.folder, &slot.current)?;
            fs::rename(current, file_path(&self.folder, &slot.previous)?).map_err(|error| io_error(&error))?;
        }
        commit(&self.folder, staged)
    }
}

/// The current file of slot `number`: the only cache file written atomically (the previous one is a rename).
pub(crate) fn current_file_name(number: u8) -> String {
    format!("scan-{number}.bin")
}

fn size_of(bytes: &[u8]) -> ByteSize {
    ByteSize::new(u64::try_from(bytes.len()).unwrap_or(u64::MAX))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use fresher_core::{ByteSize, ScopeKey};
    use tempfile::TempDir;

    use crate::{CachePolicy, DataFolder, FileScanCache};

    const FIRST: ScopeKey = ScopeKey::new(1);
    const SECOND: ScopeKey = ScopeKey::new(2);
    const THIRD: ScopeKey = ScopeKey::new(3);

    fn cache(policy: CachePolicy) -> (TempDir, FileScanCache) {
        let root = TempDir::new().unwrap();
        let folder = DataFolder::open(root.path()).unwrap();
        (root, FileScanCache::with_policy(folder, policy))
    }

    fn small() -> CachePolicy {
        CachePolicy { cap: ByteSize::new(4096), body_limit: ByteSize::new(16_384), ..CachePolicy::DEFAULT }
    }

    fn names(path: &Path) -> Vec<String> {
        let mut names: Vec<String> =
            fs::read_dir(path).unwrap().map(|entry| entry.unwrap().file_name().into_string().unwrap()).collect();
        names.sort();
        names
    }

    /// Bytes deflate cannot shrink, the same on every run.
    fn incompressible(length: usize) -> Vec<u8> {
        let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
        (0..length)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                state.to_le_bytes()[0]
            })
            .collect()
    }

    mod negative {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        use fresher_core::{ByteSize, ScanCache, StoreError};

        use super::{FIRST, SECOND, cache, incompressible, names, small};
        use crate::envelope::{FORMAT_VERSION, seal};

        #[test]
        fn a_body_over_the_cap_is_refused_and_nothing_is_written() {
            let (root, cache) = cache(small());

            let result = cache.save(FIRST, &incompressible(4096));

            assert!(
                matches!(result, Err(StoreError::TooLarge { cap, .. }) if cap == ByteSize::new(4096)),
                "{result:?}"
            );
            assert_eq!(names(root.path()), Vec::<String>::new());
        }

        #[test]
        fn a_body_over_the_body_limit_is_refused_even_when_it_compresses() {
            let (root, cache) = cache(small());

            let result = cache.save(FIRST, &vec![0_u8; 16_385]);

            assert_eq!(result, Err(StoreError::TooLarge { size: ByteSize::new(16_385), cap: ByteSize::new(16_384) }));
            assert_eq!(names(root.path()), Vec::<String>::new());
        }

        #[test]
        fn a_flipped_byte_is_discarded() {
            let (root, cache) = cache(small());
            cache.save(FIRST, b"body").unwrap();
            let path = root.path().join("scan-1.bin");
            let mut bytes = fs::read(&path).unwrap();
            let last = bytes.len() - 1;
            bytes[last] ^= 0x01;
            fs::write(&path, bytes).unwrap();

            assert_eq!(cache.load(FIRST), Ok(None));
        }

        #[test]
        fn a_truncated_file_is_discarded() {
            let (root, cache) = cache(small());
            cache.save(FIRST, b"body").unwrap();
            let path = root.path().join("scan-1.bin");
            let whole = fs::read(&path).unwrap();

            for length in 0..whole.len() {
                fs::write(&path, &whole[..length]).unwrap();

                assert_eq!(cache.load(FIRST), Ok(None), "{length}");
            }
        }

        #[test]
        fn a_body_that_inflates_past_the_cap_is_discarded() {
            let (root, cache) = cache(small());
            fs::write(root.path().join("scan-1.bin"), seal(FIRST, &vec![0_u8; 16_385])).unwrap();

            assert_eq!(cache.load(FIRST), Ok(None));
        }

        #[test]
        fn another_scopes_file_is_not_loaded() {
            let (_root, cache) = cache(small());
            cache.save(FIRST, b"first").unwrap();

            assert_eq!(cache.load(SECOND), Ok(None));
            assert_eq!(cache.load_previous(SECOND), Ok(None));
        }

        #[test]
        fn a_taken_over_slot_does_not_hand_out_the_previous_scopes_file() {
            let (root, cache) = cache(small());
            // Slot 1 was SECOND's, then FIRST took it over: its previous file is still SECOND's.
            fs::write(root.path().join("scan-1.previous.bin"), seal(SECOND, b"second")).unwrap();
            fs::write(root.path().join("scan-1.bin"), seal(FIRST, b"first")).unwrap();

            assert_eq!(cache.load(FIRST), Ok(Some(b"first".to_vec())));
            assert_eq!(cache.load_previous(FIRST), Ok(None));
        }

        #[test]
        fn a_newer_format_is_discarded_and_left_in_place() {
            let (root, cache) = cache(small());
            let path = root.path().join("scan-1.bin");
            let mut newer = seal(FIRST, b"from a newer app");
            newer[8..10].copy_from_slice(&(FORMAT_VERSION + 1).to_le_bytes());
            fs::write(&path, &newer).unwrap();

            assert_eq!(cache.load(FIRST), Ok(None));
            cache.save(SECOND, b"second").unwrap();

            assert_eq!(fs::read(&path).unwrap(), newer);
            assert_eq!(cache.load(SECOND), Ok(Some(b"second".to_vec())));
        }

        #[test]
        fn a_failed_write_keeps_the_current_generation() {
            let (root, cache) = cache(small());
            cache.save(FIRST, b"older").unwrap();
            cache.save(FIRST, b"current").unwrap();
            fs::set_permissions(root.path(), fs::Permissions::from_mode(0o500)).unwrap();

            let result = cache.save(FIRST, b"new");

            fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
            assert!(matches!(result, Err(StoreError::Io { .. })), "{result:?}");
            assert_eq!(cache.load(FIRST), Ok(Some(b"current".to_vec())));
            assert_eq!(cache.load_previous(FIRST), Ok(Some(b"older".to_vec())));
            assert_eq!(names(root.path()), ["scan-1.bin", "scan-1.previous.bin"]);
        }

        #[test]
        fn the_store_never_calls_a_deleting_function() {
            let (root, cache) = cache(small());

            for round in 0_u8..20 {
                for scope in [FIRST, SECOND, super::THIRD] {
                    cache.save(scope, &[round; 64]).unwrap();
                    cache.save(scope, &[round; 32]).unwrap();
                }
            }

            assert_eq!(names(root.path()), ["scan-1.bin", "scan-1.previous.bin", "scan-2.bin", "scan-2.previous.bin"]);
        }

        #[test]
        fn nothing_saved_loads_as_nothing() {
            let (_root, cache) = cache(small());

            assert_eq!(cache.load(FIRST), Ok(None));
            assert_eq!(cache.load_previous(FIRST), Ok(None));
        }

        #[test]
        fn a_slot_that_cannot_be_read_is_an_error() {
            let (root, cache) = cache(small());
            fs::write(root.path().join("scan-1.bin"), b"").unwrap();
            fs::set_permissions(root.path().join("scan-1.bin"), fs::Permissions::from_mode(0o000)).unwrap();

            let result = cache.load(FIRST);

            assert!(matches!(result, Err(StoreError::Io { .. })), "{result:?}");
        }
    }

    mod positive {
        use std::fs::{self, File};
        use std::os::unix::fs::MetadataExt;
        use std::time::{Duration, SystemTime};

        use fresher_core::{ByteSize, ScanCache, SettingsStore, StoredSettings};

        use super::{FIRST, SECOND, THIRD, cache, incompressible, small};
        use crate::{CachePolicy, DataFolder, FOOTPRINT_BUDGET, FileScanCache, FileSettingsStore};

        fn set_modified(path: &std::path::Path, seconds: u64) {
            let file = File::options().write(true).open(path).unwrap();
            file.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)).unwrap();
        }

        #[test]
        fn the_default_policy_is_two_scopes_of_eight_mebibytes() {
            assert_eq!(CachePolicy::DEFAULT.scopes.get(), 2);
            assert_eq!(CachePolicy::DEFAULT.cap, ByteSize::new(8 * 1024 * 1024));
            let root = tempfile::TempDir::new().unwrap();

            let cache = FileScanCache::new(DataFolder::open(root.path()).unwrap());

            assert_eq!(cache.cap(), ByteSize::new(8 * 1024 * 1024));
        }

        #[test]
        fn save_then_load_returns_the_body() {
            let (_root, cache) = cache(small());

            cache.save(FIRST, b"the body").unwrap();

            assert_eq!(cache.load(FIRST), Ok(Some(b"the body".to_vec())));
            assert_eq!(cache.load_previous(FIRST), Ok(None));
        }

        #[test]
        fn a_compressible_body_is_stored_smaller() {
            let (root, cache) = cache(small());
            let body = b"Library/Caches/".repeat(1000);

            cache.save(FIRST, &body).unwrap();

            assert!(fs::metadata(root.path().join("scan-1.bin")).unwrap().len() < 1000);
            assert_eq!(cache.load(FIRST), Ok(Some(body)));
        }

        #[test]
        fn a_second_save_moves_the_first_to_previous() {
            let (_root, cache) = cache(small());

            cache.save(FIRST, b"first").unwrap();
            cache.save(FIRST, b"second").unwrap();

            assert_eq!(cache.load(FIRST), Ok(Some(b"second".to_vec())));
            assert_eq!(cache.load_previous(FIRST), Ok(Some(b"first".to_vec())));
        }

        #[test]
        fn a_third_scope_takes_the_oldest_slot() {
            let (root, cache) = cache(small());
            cache.save(FIRST, b"first").unwrap();
            cache.save(SECOND, b"second").unwrap();
            set_modified(&root.path().join("scan-1.bin"), 2_000);
            set_modified(&root.path().join("scan-2.bin"), 1_000);

            cache.save(THIRD, b"third").unwrap();

            assert_eq!(cache.load(FIRST), Ok(Some(b"first".to_vec())));
            assert_eq!(cache.load(SECOND), Ok(None));
            assert_eq!(cache.load(THIRD), Ok(Some(b"third".to_vec())));
            assert_eq!(fs::read(root.path().join("scan-2.bin")).unwrap()[10..18], THIRD.get().to_le_bytes());
        }

        #[test]
        fn the_footprint_stays_within_the_policy() {
            let root = tempfile::TempDir::new().unwrap();
            let folder = DataFolder::open(root.path()).unwrap();
            FileSettingsStore::new(folder.clone()).save(&StoredSettings::default()).unwrap();
            let cache = FileScanCache::new(folder.clone());
            let cap = CachePolicy::DEFAULT.cap.bytes();
            // Deflate adds about 0.1 % to bytes it cannot shrink: the sealed file lands just under the cap.
            let body = incompressible(usize::try_from(cap).unwrap() - 16 * 1024);

            for scope in [FIRST, SECOND] {
                cache.save(scope, &body).unwrap();
                cache.save(scope, &body).unwrap();
            }

            let settings = fs::metadata(root.path().join("settings.json")).unwrap().blocks() * 512;
            let footprint = folder.footprint().unwrap();
            assert_eq!(footprint.files, 5);
            assert!(footprint.total.bytes() <= 4 * cap + settings, "{footprint:?}");
            assert!(!footprint.over());
            assert!(footprint.total <= FOOTPRINT_BUDGET);
        }
    }
}
