# 017/01/04 — Places outside the scan roots

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
core, scanner, engine, cleaner, app/ui

Design section 3. The scan roots are the home by default, and the two temporary places lie outside it. A place is a
known folder that a registered module's interest names and no scan root holds; the walk reads it as a root of its own,
for that module's interests only.

## Files

- `crates/core/src/walk.rs` — `WalkRequest.places: &[Place]`, `Place { folder: KnownFolder, path: PathBuf }` (the
  canonical path); `NotScanned::OthersFolder` ("a folder of another user, in a place outside the scan roots").
- `crates/engine/src/` (the scan's request) — the places: every `KnownFolder` of a registered module's
  `Pattern::Under` interest, enabled or not, resolved through `KnownFolders`, that is not inside a scan root and is
  not protected `never` (so `UserTemp` and `SharedTemp` today; `Home`'s folders are inside the home root). Sorted, so
  the request is the same every scan.
- `crates/scanner/src/scanner.rs` — `prepare` adds the places to the roots, each marked a place: never another volume
  than its own, links not followed, a folder whose owner is not the scan's user is listed as itself (its sizes
  incomplete) and not entered (`OthersFolder`); `<UserTemp>/TemporaryItems` joins the standing avoided places
  (`NotScanned::Avoided`), so the privacy protection never counts as unreadable. Only interests anchored in the place
  match inside it. The cache's fingerprint includes the places (`OtherRoots` when they change).
- `crates/engine/src/cleanup/context.rs` — the cleaner's roots are the scan roots and the places of the last scan (its
  check 2); `crates/core/src/cleanup/port.rs`'s doc follows.
- `crates/engine/src/storage/` — each place is a by-place root after the scan roots, named by its known folder
  (`core.place.user-temp.name` "Your temporary folder", `core.place.shared-temp.name` "Shared temporary folder", in
  `crates/core/locales/en.json`); the by-meaning groups count its findings; the parts still add up to the volume.
- `app/ui/src/screens/overview/` — a place root's row shows its name from the text, not its path's last name; the
  bindings regenerated.

## Produces

The two places walked on every scan while the temp module is registered, their items reachable by the cleaner, shown
in the map; for 02/03 and the e2e.

## Checks

- Negative first: no module naming `SharedTemp` → no place is walked and an item there is refused by the cleaner
  (outside every root); a place inside a scan root is not walked twice; a folder of another user inside a place is not
  entered and the scan stays complete; `TemporaryItems` is counted avoided, never unreadable; a link in a place is not
  followed; a module's switch off changes nothing the walk reads.
- Positive: a fixture with the home and a place side by side — the module's interest matches inside the place, the
  cleaner moves an item from it to the Trash, the map lists the place after the home with its name and its parts add
  up; a rescan with the same places uses the cache.
- `cargo nextest run -p fresher-core -p fresher-scanner -p fresher-engine -p fresher-cleaner`; `npm run sandbox:test`
  (the existing goldens unchanged: no existing module names a place); `npx vitest run src/screens/overview/`.

## Conclusions

Built (15f2600, review fixes 5c4e525): `Place`, `WalkRequest.places`, `NotScanned::OthersFolder`, the engine's
`places.rs` (every `KnownFolder::is_walked_place` folder a registered module's interest names, enabled or not, located,
sorted), places as roots after the scan roots in the scanner and the reference walk, only anchored interests matching in
them, the cleaner's roots (the scan roots and the last scan's places), the map's place roots with their names, and the
overview's root list naming a place by its text.

Changed from the brief, each a ruling:

- A place must also lie outside the home: narrowing the scan roots to `~/work` had turned `Home` into a place.
- The walked places are always read from the disk (decision 21, from 01/03's review): a rescan drops every journal
  change under a place and walks each place again in full; a restore emits nothing inside a place but keeps their cached
  sizes for the map.
- A place that is missing or cannot be read is left out of the walk and counted not scanned, unreadable; it never stops
  the scan or a restore. The places are not in the cache's key; changed places give `OtherRoots`.
- A place that holds a scan root or lies inside one is not walked, instead of refusing such roots in the settings
  (01/01's fixtures rely on them). Cost: temporary items under such a root are not offered.
- A folder of another user in a place is still matched: listed as itself with partial sizes and an inside that is
  `foreign` and not `complete`, so the module can leave it out. `TemporaryItems` is `NotScanned::SystemItems`, never a
  reason to ask for Full Disk Access. New tree flags (bits 13 and 14); the tree format stays 4.
- The engine skips a place the cleaner's protection refuses for ever, through a new `Cleaner::protection()`.

Checks: `walk/places_tests.rs` (the always-rewalk verified to fail without it), the engine's and the reference walk's
tests, and `crates/e2e/tests/places.rs` (five flows, and an item of a place no module names refused: a debug build
answers `NotInSandbox` before `OutsideRoots`, a release build `OutsideRoots`). No screenshot changed: no mock had places
until 03/03.
