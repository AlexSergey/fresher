# 004/02/03 — Disk state and the cache port

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** core, testing

## Files

- `crates/core/src/disk_state.rs` (new) — the read-only tree of the last committed scan:

  ```rust
  pub trait DiskState: Send + Sync {
      fn id(&self) -> StateId;
      fn taken(&self) -> Timestamp;
      fn roots(&self) -> Vec<NodeId>;
      fn node(&self, id: NodeId) -> Option<NodeInfo>;
      fn children(&self, id: NodeId, order: ChildOrder, offset: u32, limit: u32) -> Option<NodePage>;
      fn largest_files(&self, limit: u32) -> Vec<NodeInfo>;
      fn search(&self, query: &str, limit: u32) -> Vec<NodeInfo>;
      fn unreadable(&self, limit: u32) -> Vec<UnreadablePlace>;
      fn path(&self, id: NodeId) -> Option<PathBuf>;
  }
  pub struct NodeId { state: StateId, index: u32 }      // refused by another state, like a FindingId
  pub struct NodeInfo { pub id: NodeId, pub name: String, pub kind: NodeKind, pub sizes: Sizes,
                        pub files: u64, pub folders: u64, pub owner: Option<CategoryId>,
                        pub modified: Option<Timestamp>, pub not_scanned: Option<NotScanned> }
  pub enum NodeKind { Folder, File, Link, Collapsed }
  pub enum ChildOrder { BySize, ByName }
  pub enum NotScanned { Unreadable(Unreadable), Avoided, OtherVolume }
  pub struct UnreadablePlace { pub node: NodeInfo, pub reason: NotScanned, pub hidden_folders: u64 }
  ```

  `NodeInfo` and the types it carries derive `Serialize` and `TS` (the commands of plans 003 and 012 return them);
  `name` is lossy UTF-8 for display, the path stays bytes inside the scanner.
- `crates/core/src/scan_cache.rs` (new) — the cache of a scan scope, implemented by the store:

  ```rust
  pub trait ScanCache: Send + Sync {
      /// The largest body `save` accepts.
      fn cap(&self) -> ByteSize;
      /// The body of the current generation for `scope`; `None` when there is none or it was discarded.
      fn load(&self, scope: &ScopeKey) -> Result<Option<Vec<u8>>, StoreError>;
      /// The body of the previous generation (for plan 010).
      fn load_previous(&self, scope: &ScopeKey) -> Result<Option<Vec<u8>>, StoreError>;
      /// Makes `body` the current generation of `scope`; the current one becomes the previous.
      fn save(&self, scope: &ScopeKey, body: &[u8]) -> Result<(), StoreError>;
  }
  pub struct ScopeKey(u64);   // a stable hash of the canonical roots and their volume ids
  ```

  `StoreError` gains `TooLarge { size, cap }`; its messages name "the file" instead of "the settings file" (the
  conclusion of plan 002/03 left this to the cache).
- `crates/core/src/error.rs`, `crates/core/locales/en.json` — `CommandError::NoScan` ("Scan first to see this.") and
  `CommandError::StaleNode` ("This view is out of date: the scan results changed. Open it again."), for the engine's
  queries (08/02); the core's dictionary test covers both.
- `crates/core/src/hash.rs` (new) — `stable_hash(bytes) -> u64`, FNV-1a 64, with a test vector; used for `ScopeKey`
  and the interest fingerprint (04/02). `std`'s hasher is not stable across Rust versions, so it is not used.
- `crates/testing/src/fakes.rs` — `MemoryScanCache` (two generations per scope in memory, a configurable cap) and
  `FixedDiskState` (a small tree from a list of paths and sizes) for the engine's tests.

## Produces

`DiskState` for the scanner (08) and the engine (05, 07, 08); `ScanCache` for the store (06/01) and the scanner
(06/02, 07); `stable_hash` for the matcher (04/02) and the scope key (05).

## Checks

- Negative: `a_node_id_of_another_state_is_refused` (via `FixedDiskState`), `the_memory_cache_refuses_a_body_over_its_cap`,
  `an_empty_scope_loads_nothing`.
- Positive: `stable_hash_matches_the_fnv_1a_test_vectors`, `save_moves_the_current_generation_to_previous`,
  `node_info_serializes_in_camel_case`.
- `cargo nextest run -p fresher-core -p fresher-testing`; the bindings test lists the new IPC types (they are not used
  by a command yet; the test's list is extended so they are generated and checked).

## Conclusions

- `DiskState`, `NodeId`, `NodeInfo`, `NodeKind`, `ChildOrder`, `NotScanned` and `UnreadablePlace` are in
  `crates/core/src/disk_state.rs` as written. Not spelled out by the step, taken the simplest way:
  - `StateId(u32)` with `new`/`get`, like `ScanId`; `NodeId::new(state, index)` with `state()` and `index()`;
    `NodeId`, `StateId`, `NodeKind` and `ChildOrder` also derive `Deserialize` (the UI passes them back to commands).
  - `NodePage { nodes, offset, total }`, shaped like `FindingsPage`.
  - `NotScanned` serializes as `{ kind, value }` (like `Subject`); for that `Unreadable` now derives `Serialize` and
    `TS` (kebab-case). It is still named `needs-full-disk-access` until step 04 renames it `guarded`.
  - `files` and `folders` count everything inside a folder, at every depth; `hidden_folders` counts the folders under
    a not-scanned folder.
- **Deviation: `ScanCache` takes `ScopeKey` by value**, not `&ScopeKey`: the key is an 8-byte `Copy` value and Clippy
  (`trivially_copy_pass_by_ref`, denied in the workspace) refuses the reference. Plans 06 and 07 call it with a value.
  `ScopeKey` has `new(u64)` and `get()`; computing it from the roots belongs to the scanner (05).
- `StoreError::TooLarge { size, cap }` (both `ByteSize`); every `StoreError` message now names "the file". The engine
  maps `TooLarge` from the settings store (which never returns it) to `SettingsFile::Unreadable`, like `Io`.
- `CommandError::NoScan` (`no-scan`) and `CommandError::StaleNode` (`stale-node`) with their English texts; the
  error and dictionary tests list them.
- `stable_hash` is FNV-1a 64, checked against the published vectors for "", "a" and "foobar".
- `MemoryScanCache::with_cap(cap)` is in `fakes.rs`. **`FixedDiskState` got its own file**,
  `crates/testing/src/fixed_disk_state.rs`, instead of `fakes.rs`, which would have passed 500 lines.
  `FixedDiskState::new(id, taken, [(path, kind, sizes)])` takes nodes by path (a node without a parent node is a
  root; sizes are taken as given), and `with_owner` / `with_not_scanned` record a claim or a reason on a path.
- The bindings test lists the disk state's types; the bindings were regenerated.
