# 004/02/02 — The OS ports

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** core, testing

The three ports through which the scanner reaches the operating system, in OS-neutral words. No IO in the core.

## Files

- `crates/core/src/folder.rs` (new) — reading one folder:

  ```rust
  pub trait FolderReader: Send + Sync {
      /// The identity of a root folder; never follows a link.
      fn identify(&self, path: &Path) -> Result<FolderIdentity, ReadError>;
      /// Every entry of the folder at `path`, which must still be `expected`; names go into `names`.
      fn read(&self, path: &Path, expected: FolderIdentity, out: &mut FolderListing) -> Result<(), ReadError>;
      /// The first `limit` bytes of the regular file `name` in the folder; never follows a link.
      fn head(&self, folder: &Path, name: &str, limit: usize) -> Result<Vec<u8>, ReadError>;
  }
  pub struct FolderIdentity { pub device: u64, pub file_id: u64 }
  pub struct FolderListing { /* entries + one name buffer, reused between calls */ }
  pub struct RawEntry { pub kind: EntryKind, pub device: u64, pub file_id: u64, pub links: u32,
                        pub sizes: Sizes, pub clone: Option<u64>, pub modified: Option<Timestamp>,
                        pub cloud_only: bool, pub error: Option<ReadError> }
  pub enum ReadError { NoPermission, Guarded, Vanished, Replaced, Interrupted, Failed }
  ```

  `FolderListing` offers `clear()`, `push(name: &[u8], entry)` and an iterator of `(&[u8], &RawEntry)`, so a reader
  fills it with no allocation per entry. Names are bytes (a file name is not always UTF-8).
- `crates/core/src/journal.rs` (new) — the change journal:

  ```rust
  pub trait ChangeJournal: Send + Sync {
      /// Where the journal of `volume` stands now; `None` when the volume keeps no journal.
      fn position(&self, volume: &VolumeId) -> Option<JournalPosition>;
      /// The folders changed on `volume` since `since`, up to now.
      fn changes_since(&self, volume: &VolumeId, since: &JournalPosition, cancel: &CancelToken)
          -> Result<Changes, JournalGap>;
  }
  pub struct JournalPosition { pub journal: u128, pub cursor: u64 }
  pub struct Changes { pub folders: Vec<PathBuf>, pub subtrees: Vec<PathBuf>, pub end: JournalPosition }
  pub enum JournalGap { NoJournal, Reset, Dropped, Wrapped, RootChanged, TooOld, Cancelled, Failed }
  ```

- `crates/core/src/volume.rs` (new) — the volume facts the scan needs:

  ```rust
  pub trait Volumes: Send + Sync {
      fn identify(&self, path: &Path) -> Result<VolumeFacts, ReadError>;
  }
  pub struct VolumeId(u128);           // stable across reboots (the volume's UUID on macOS)
  pub struct VolumeFacts { pub id: VolumeId, pub device: u64, pub case_sensitive: bool, pub used: ByteSize }
  ```

  Plan 009 extends this port (kinds, free and purgeable space); the methods above stay.
- `crates/core/src/lib.rs` — the new modules and their `pub use`.
- `crates/testing/src/fakes.rs` — `FakeVolumes` (fixed facts per root), `FakeChangeJournal` (a script: positions and
  the `Changes` or `JournalGap` each `changes_since` returns; records the calls). `FolderReader` needs no fake here:
  the scanner's portable `StdReader` (05/02) is the test reader, and a counting wrapper is written there.

## Produces

The ports 03 implements for macOS, 05 and 07 use in the scanner, and the fakes 07 drives the rescan logic with.

## Checks

- Negative: `a_listing_refuses_a_name_with_a_separator_or_a_nul` (`push` returns an error, never stores it),
  `the_fake_journal_returns_its_scripted_gap`.
- Positive: `a_listing_is_reused_without_reallocating` (capacity after `clear` kept),
  `the_fake_journal_records_each_call`, `the_fake_volumes_answer_per_root`.
- `cargo nextest run -p fresher-core -p fresher-testing`; `cargo clippy --workspace --all-targets -- -D warnings`.
- Every new public item is documented in one line; the core still has no IO (no `std::fs` in `crates/core/src`).

## Conclusions

- The three ports are in `fresher-core` as written: `FolderReader` (`folder.rs`), `ChangeJournal` (`journal.rs`) and
  `Volumes` (`volume.rs`); the core still has no `std::fs`.
- Unspecified details, taken the simplest way:
  - `FolderListing::push` refuses a name that is empty, `.`, `..`, or holds `/` or NUL, with `ReadError::Failed`
    (a malformed record fails the folder, as the design says), so the reader can write `listing.push(name, entry)?`.
    The listing also has `new`, `len` and `is_empty`. Names are kept as byte ranges into one buffer.
  - `RawEntry` is `Copy`; `ReadError` and `JournalGap` are `Copy` `thiserror` enums with messages that hold no path.
  - `VolumeId` has `new(u128)` and `get()`; `VolumeFacts` is `Copy`. `Changes` has no `Default` (its `end` has no
    meaningful default).
- `FakeVolumes` answers with the facts of the longest root a path starts with, and `ReadError::Vanished` outside
  every root. `FakeChangeJournal` takes positions per volume (`at`) and a queue of answers (`answering`); once the
  queue runs out it answers `JournalGap::Failed`, and a cancelled token answers `JournalGap::Cancelled`; every
  `changes_since` call is recorded as (volume, position).
