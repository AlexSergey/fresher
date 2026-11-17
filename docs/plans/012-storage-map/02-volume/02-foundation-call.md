# 012/02/02 — The Foundation call

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:** macos,
workspace

## Files

- `crates/macos/src/ffi/volume.rs` — `resources(path: &Path) -> io::Result<VolumeResources>`, with
  `VolumeResources { important_available: Option<u64>, internal: Option<bool> }`: one `NSURL` made from the path's
  bytes (`NSURL::from_path`, as `ffi::trash`), then two resource values of the volume through `getResourceValue:
  forKey:error:` — `NSURLVolumeAvailableCapacityForImportantUsageKey` (the capacity available for important usage: the
  free space plus what the system can purge) and `NSURLVolumeIsInternalKey` — each read as an `NSNumber`, each `None`
  when the system gives no value. Inside one `autoreleasepool`. Every `unsafe` block holds one operation and its
  `// SAFETY:` comment, per the `ffi` module's rule (a key static is read in its own block).
- `crates/macos/Cargo.toml` — `objc2-foundation`'s features gain `NSValue` (`NSNumber`) and whatever the resource
  value call needs beyond `NSURL`, `NSString`, `NSError` (named exactly when built).
- root `Cargo.toml` — nothing if the feature list lives in the crate; `Cargo.lock` regenerated.
- `docs/dependencies.md` — the `objc2-foundation` row's features, if the table lists them; no new crate, no new
  licence.

## Produces

The two values `MacVolumes::space` (03) needs that `statfs` does not give.

## Checks

- Negative first: `a_path_that_does_not_exist_is_an_error` (`NotFound`), `a_path_with_a_nul_is_refused`
  (`InvalidInput`, as `c_path`).
- Positive: `the_important_capacity_is_at_least_the_free_space` (on a temporary folder, against
  `statfs.f_bavail × f_bsize` read just before; the purgeable answer changes from moment to moment, so only this
  bound is checked, as the deferred item says), `the_volume_of_a_temporary_folder_is_internal` (skipped with a printed
  reason on a Mac whose temporary folder is not on an internal disk), `a_disk_image_is_not_internal` (`DiskImage::mount`,
  skipped without `hdiutil`).
- `cargo nextest run -p fresher-macos`; `npm run check:crate-deps`; `cargo deny check`.

## Conclusions

Built in `3eb972f`, minors fixed in `1124b6d`, reviewed approved.

- One `NSURL` call reads the important free capacity and the internal flag; one `unsafe` operation per block, each
  with a `SAFETY` comment. The only manifest change is the `NSValue` feature of `objc2-foundation`.
- Deviation: on a mounted disk image the system gives no value for the internal flag (`None`, not `false`), so the
  test asserts `None | Some(false)` and 02/03 reads `None` as not known internal.
- Measured: `cargo nextest run -p fresher-macos` 131 passed, 1 skipped (no disk image tool).
