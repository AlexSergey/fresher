# 004/06/01 — Cache files in the store

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** store

## Files

- `crates/store/Cargo.toml` — dependency `crc32fast` (exact version, the one already in `Cargo.lock`; MIT OR
  Apache-2.0), and `miniz_oxide` (exact version, the one already in `Cargo.lock`; MIT OR Zlib OR
  Apache-2.0) for the body's compression (the user's decision of 2026-11-15).
- `crates/store/src/envelope.rs` — the binary envelope, the store's own: `magic = b"FRSHSCAN"`, `format_version: u16`
  (1), `scope: u64`, `body_len: u64`, `crc32: u32` of the stored bytes, then the body compressed with deflate level 1. `seal` compresses, `open` checks
  the checksum and then inflates with a size limit of the cap (a body that inflates past it is `Discarded`). `seal(scope, body) -> Vec<u8>` and
  `open(bytes, scope) -> Opened { Body(&[u8]), Discarded(Reason) }`: another magic, a short file, a length that does
  not match, a wrong checksum, another scope → `Discarded`; a newer `format_version` → `Discarded` too, and the file
  is not overwritten until a save of the current scope replaces its slot (the cache is rebuildable:
  `VersionPolicy::Discard`).
- `crates/store/src/scan_cache.rs` — `FileScanCache` implements the core's `ScanCache`:
  - `CachePolicy { scopes: 2, generations: 2, cap: 8 MiB }` (a `const DEFAULT`; plan 010 changes the policy, not the
    format); `cap()` returns the policy's cap;
  - slots `scan-1.bin`, `scan-1.previous.bin`, `scan-2.bin`, `scan-2.previous.bin`: a scope lives in the slot whose
    current file carries its key; a new scope takes a free slot, else the slot whose current file is the oldest
    (by the file's modification time, read through `DataFolder`, not by the clock);
  - `save(scope, body)`: refuses a body over the cap (`StoreError::TooLarge`); if the slot's current file is this
    scope's, renames it over the slot's `previous` file (`std::fs::rename`: no deleting function); then writes the
    new current file with `write_atomic`. A slot taken over by a new scope gets its `previous` file replaced by the
    first rotation; until then `load_previous` checks the scope key and answers `None` for a foreign one;
  - `load` and `load_previous`: `read_file`, `envelope::open`; a discarded file is logged with its reason (no path)
    and answers `None`.
- `crates/store/src/lib.rs` — `pub use scan_cache::{CachePolicy, FileScanCache}`.
- `docs/architecture/README.md` ("Data") — the scan cache: its slots, generations, cap, envelope and the rule that it
  never deletes; the footprint budget's share.

## Produces

The cache files for the scanner's codec (06/02) and the rescan (07); the policy plan 010 extends.

## Checks

- Negative: `a_body_over_the_cap_is_refused_and_nothing_is_written`, `a_flipped_byte_is_discarded`,
  `a_truncated_file_is_discarded` (every length from 0 to the full file), `a_body_that_inflates_past_the_cap_is_discarded`, `another_scopes_file_is_not_loaded`,
  `a_newer_format_is_discarded_and_left_in_place`, `a_failed_write_keeps_the_current_generation`,
  `the_store_never_calls_a_deleting_function` (Clippy's `disallowed-methods` already fails the build; the test lists the
  folder after many saves and finds only the four slot names and nothing else).
- Positive: `save_then_load_returns_the_body`, `a_compressible_body_is_stored_smaller`, `a_second_save_moves_the_first_to_previous`,
  `a_third_scope_takes_the_oldest_slot`, `the_footprint_stays_within_the_policy` (fill every slot to its cap:
  `DataFolder::footprint()` ≤ 4 × cap + the settings file).
- `cargo nextest run -p fresher-store`; coverage ≥ 90 %; `cargo deny check`.

## Conclusions

- `FileScanCache` (`crates/store/src/scan_cache.rs`) implements `ScanCache` on the slots `scan-1.bin`,
  `scan-1.previous.bin`, `scan-2.bin` and `scan-2.previous.bin`; `crates/store/src/envelope.rs` seals and opens the
  files. `crc32fast` 1.5.5 and `miniz_oxide` 0.9.1 (the newer of the two in `Cargo.lock`) are workspace dependencies;
  `cargo deny check` passes.
- **The envelope**, little-endian: `FRSHSCAN`, `format_version: u16` (1), `scope: u64`, `stored_len: u64` (the
  length of the compressed bytes), `crc32: u32` of the compressed bytes — 30 bytes — then the body as a raw deflate
  stream, level 1. `open` checks, in order: the magic, the version, the length, the checksum, the scope, then
  inflates. Every failure is a `Reason` (`Short`, `NotACacheFile`, `NewerFormat`, `OlderFormat`, `LengthMismatch`,
  `Checksum`, `OtherScope`, `NotDeflate`, `TooLarge`), logged without a path.
- **Deviation: the cap is on the file, a separate limit on the body.** The step says both "refuses a body over the
  cap" and "inflates with a size limit of the cap", while the design (section 10, the user's decision) says the cap
  applies to the file on disk, so that a 13.6 MB home body, 5.5 MB compressed, fits without collapsing. Following the
  design: `save` refuses a sealed file over `cap` (8 MiB), and `CachePolicy` has a `body_limit` (64 MiB) that bounds
  the body before compression, on save and as the inflate limit on load (a forged or broken file cannot make a load
  take more memory). `cap()` returns the file cap; a body at or under it always fits unless it is incompressible
  (deflate adds about 0.1 % then), in which case `save` answers `TooLarge` and the next scan is a full one. Step 03
  may compare the compressed size with the cap to collapse less.
- **Deviation: no `generations` field in `CachePolicy`.** The port fixes two generations (`load`,
  `load_previous`), so a field that could not change would mislead: `CachePolicy { scopes: NonZeroU8, cap,
  body_limit }`, `DEFAULT` = 2 scopes, 8 MiB, 64 MiB; `FileScanCache::new` takes the default, `with_policy` another.
- **Deviation: `open` returns an owned body** (`Opened::Body(Vec<u8>)`): inflating makes a new buffer.
- A slot belongs to a scope when the header of its current file (read alone, 30 bytes) is of the current version and
  names the scope; a newer file's scope is not read, so its slot is only taken as a free or the oldest slot.
- **A failed write keeps both generations**: `atomic.rs` was split into `stage` (temporary file written and synced)
  and `commit` (rename over the target, folder synced), and `write_atomic` is the two together. `save` stages the
  new file first, then renames the current file over the previous one, then commits; a failure while staging changes
  nothing.
- A slot taken over by a new scope does not rotate (the old current file is not this scope's); its previous file
  stays the other scope's until the first rotation and `load_previous` answers `None` for it.
- Tests: the step's checks, plus every flipped byte and every truncation of the envelope, a body over the body limit,
  a stream that is not deflate, and an unreadable slot. The footprint test fills the four slots with incompressible
  bodies just under 8 MiB: the folder stays within 4 × cap + the settings file and within `FOOTPRINT_BUDGET`.
- **Not done here: `docs/dependencies.md`** (the inventory) is not on this branch yet (it was added on `main` after
  the branch point). When it comes in, add to "Shipped with the app": `crc32fast` 1.5.5, MIT OR Apache-2.0, store;
  `miniz_oxide` 0.9.1, MIT OR Zlib OR Apache-2.0, store.
