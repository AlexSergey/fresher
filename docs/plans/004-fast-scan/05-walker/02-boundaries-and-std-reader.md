# 004/05/02 — Roots, boundaries and the portable reader

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-16 · **Modules:** scanner, testing

## Files

- `crates/scanner/src/walk/std_reader.rs` — `StdReader` implements `FolderReader` with `std::fs` only
  (`read_dir`, `symlink_metadata`, `MetadataExt` for device, file id, links, blocks): the reader of the tests, and the
  fallback on any system without its own adapter. Its open is checked like the macOS one (`O_NOFOLLOW |
  O_DIRECTORY`, then device and file id against `expected`); `reclaimable = allocated`; `head` as in 03/02.
- `crates/scanner/src/walk/counting.rs` — `CountingReader<R>`: wraps a reader and counts `read` calls per folder;
  the tool of every deterministic "folders read" bar (07/03).
- `crates/scanner/src/walk/roots.rs` — `Roots::prepare(roots, volumes: &dyn Volumes, reader: &dyn FolderReader) ->
  Result<Roots, WalkError>`: each root canonicalized (`std::fs::canonicalize` of the root only, never of a path under
  it), identified (`FolderReader::identify`), its volume identified (`Volumes::identify`); a root inside another is
  dropped and counted in the summary; a root that is not a readable folder is `RootUnreadable { index, kind }`.
  `Roots::scope_key()` — the stable hash of the canonical roots and their volume ids (for the cache, 06).
- `crates/scanner/src/walk/boundary.rs` — `Boundary`: decides for each subfolder of a listing whether it is entered:
  not on the root's device → `NotScanned::OtherVolume`; in the avoid list → `NotScanned::Avoided`; a protected path
  (`/System`, `/usr` but `/usr/local`, `/bin`, `/sbin`, `/private/var/db`, another user's home under `/Users`) → not
  entered and not reported (the settings check already keeps roots away from them; this is the second guard). The
  avoid list and the protected list are compared as canonical paths, by components, never as string prefixes.
- `crates/testing/src/fixture_tree.rs` — fixture homes gain `[[hardlink]] path, target` (the walker's link tests and
  the generator need it); `FixtureTree::mount_point` is not added (another volume is tested with a disk image in the
  macOS crate's tests only).

## Produces

`StdReader` and `CountingReader` for every scanner test; `Roots` and `Boundary` for the workers (03) and the scope
key for the cache (06/02, 07/01).

## Checks

- Negative: `a_root_inside_another_root_is_dropped_and_counted`, `a_root_that_is_a_file_is_refused`,
  `a_root_that_is_a_link_is_refused`, `an_avoided_folder_is_not_entered` (the counting reader saw no `read` of it),
  `a_folder_on_another_device_is_not_entered` (a fake `Volumes`/reader pair that reports another device),
  `an_avoid_entry_that_is_a_prefix_string_does_not_avoid_its_neighbour` (`/a/Desk` does not avoid `/a/Desktop`),
  `the_std_reader_refuses_a_folder_swapped_for_a_link`.
- Positive: `the_std_reader_lists_a_folder_with_its_sizes`, `the_std_reader_reports_hard_links`,
  `the_scope_key_does_not_depend_on_root_order`, `the_scope_key_changes_with_the_roots`.
- `cargo nextest run -p fresher-scanner -p fresher-testing`.

## Conclusions

- `StdReader` and `CountingReader` are public (the e2e tests and the benches use them); `Roots` and `Boundary` are
  crate-private. `fresher-scanner` takes `libc` (already a workspace dependency) for the open flags and error numbers.
- `StdReader` opens the folder with `O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC` and checks its identity through the handle,
  then lists it with `read_dir` and one `symlink_metadata` per entry (`std` cannot list through a handle). Taken the
  simplest way: the identity is checked again on the path after the listing (a swap during the listing is
  `Replaced`), and a handle whose link count is 0 is `Vanished`. Error numbers map as in the macOS reader; a folder's
  own size is zero, a link's is its own `lstat` size, every file's reclaimable is its allocated; an entry whose
  metadata fails is listed with its error.
- `CountingReader` keeps the order of the reads (`log`), and counts per folder (`reads`) and in all (`total`).
- `Roots::prepare`: a root that is a link or not a folder is `RootUnreadable { kind: NotADirectory }`; a refusal of the
  reader or the volumes is mapped to an I/O kind. **Deviation:** "dropped and counted in the summary" — `WalkSummary`
  (frozen core) has no field for it, so the count is kept in `Roots::dropped()` and logged; the same root given twice
  counts as dropped once. `scope_key` hashes the sorted (path, volume id) pairs, length-prefixed.
- `Boundary` decides in the order: protected (not entered, not reported) → another device (`OtherVolume`) → avoided.
  Another user's home is a folder right under `/Users` that holds no root. The avoid list is canonicalized once when
  its places exist and compared by `Path` equality (components).
- `FixtureTree` takes `[[hardlink]] path, target` (both inside the root; documented in `docs/testing.md`).
- Two checks of the step need the walk and are in 03's tests: `an_avoided_folder_is_not_entered` with the counting
  reader and `a_folder_on_another_device_is_not_entered` with a reader that reports another device; here they are
  checked on `Boundary` itself.
