# 004/03/02 — The bulk read

**Status:** ✔️ done · **Version:** 0.3.0 · **Date:** 2026-11-15 · **Modules:** macos

## Files

- `crates/macos/src/ffi/bulk.rs` — `getattrlistbulk(fd, &attrlist, buf, len, FSOPT_PACK_INVAL_ATTRS |
  FSOPT_ATTR_CMN_EXTENDED | FSOPT_NOFOLLOW)` on a file descriptor the safe code opened; the attribute list from 01:
  `ATTR_CMN_RETURNED_ATTRS | NAME | DEVID | OBJTYPE | FILEID | MODTIME | FLAGS | ERROR`, `ATTR_FILE_LINKCOUNT |
  TOTALSIZE | ALLOCSIZE`, `ATTR_CMNEXT_CLONEID | PRIVATESIZE`. The only unsafe part is the call; the buffer is a
  `Vec<u8>` the caller owns, its length passed exactly.
- `crates/macos/src/bulk/parse.rs` — the record parser, safe: reads lengths and fields with bounds checks
  (`get(..)` and `from_ne_bytes`), honours the returned-attributes mask, the 4-byte packing and 8-byte record
  alignment, the name's `attrreference`, the signed device id; turns each record into a `RawEntry` pushed into the
  `FolderListing`. A record that does not fit its length, a name that runs out of the buffer or contains a `/` or a NUL
  fails the folder as `ReadError::Failed`.
- `crates/macos/src/bulk/reader.rs` — `MacFolderReader` implements `FolderReader`:
  - `read` opens the folder with `OpenOptions` and `custom_flags(O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC)` (safe std
    calls), `fstat`s the handle through `File::metadata` and refuses a device or file id other than `expected`
    (`Replaced`), then calls the bulk read until it returns 0 entries; `EINTR` is retried up to three times.
  - The `errno` mapping: `EPERM` → `Guarded`, `EACCES` → `NoPermission`, `ENOENT`/`ENOTDIR` →
    `Vanished`, `ELOOP` (a link) → `Replaced`, anything else → `Failed`.
  - A file whose flags say its data is not local (`SF_DATALESS`) gets `cloud_only = true` and no allocated size.
  - `head` opens `folder/name` with `O_RDONLY | O_NOFOLLOW | O_NONBLOCK`, checks it is a regular file through the
    handle, and reads at most `limit` bytes.
  - The buffer size from 01 (64 KiB unless measured otherwise), one buffer per reader per thread (a `thread_local`
    or a buffer passed in the listing).
- `crates/macos/src/lib.rs` — `pub use bulk::MacFolderReader`.

## Produces

The reader the walker uses in the app (05/06) and in its integration tests and benches (05/03).

## Checks

- Negative (parser, on made-up buffers): `a_record_longer_than_the_buffer_fails_the_folder`,
  `a_name_reference_outside_the_record_fails_the_folder`, `a_name_with_a_slash_fails_the_folder`,
  `an_entry_error_marks_the_entry_not_the_folder`.
- Negative (reader, on temporary folders): `a_folder_replaced_by_a_link_is_refused`,
  `a_folder_replaced_by_another_folder_is_refused`, `a_folder_with_mode_000_is_no_permission`,
  `a_vanished_folder_is_reported_vanished`, `head_does_not_follow_a_link`, `head_does_not_block_on_a_fifo`.
- Positive: `lists_the_same_entries_as_std` (names, kinds, file ids, link counts, logical and allocated sizes against
  `read_dir` + `symlink_metadata` on a fixture tree with files, folders, links, a hard link and 10 000 entries in one
  folder); `reads_a_large_folder_in_several_calls`; `a_clone_reports_its_clone_id_and_private_size`
  (`fixtures` feature).
- Property: `the_parser_never_panics_on_any_bytes` (proptest, arbitrary buffers and masks).
- `cargo nextest run -p fresher-macos`; coverage ≥ 90 %. The numbers of a single-threaded read of the `small` tree
  (05/01) are written in the notes once 05/01 exists.

## Conclusions

- `MacFolderReader` (`bulk/reader.rs`) reads a folder with `getattrlistbulk` into a 64 KiB buffer kept per thread;
  `bulk/parse.rs` parses the records with safe code (every length and offset checked, `from_ne_bytes` on copied
  fields); `ffi/bulk.rs` holds the three calls `std` lacks: `getattrlistbulk`, `getattrlistat` and `openat`.
- **Deviation (from 01's measurements, as the design's section 3 already says):** the bulk read asks for the clone id
  and the extended flags (`ATTR_CMNEXT_EXT_FLAGS`), not the private size, which doubled the walk in 01. A regular
  file flagged `EF_MAY_SHARE_BLOCKS` gets its private size by one more call (`getattrlistat` on the open folder) and
  `clone: Some(id)`; every other file has `clone: None` and reclaimable = allocated. If that call fails, the clone
  counts reclaimable = allocated (what a portable reader reports).
- Checked on this Mac: with `FSOPT_PACK_INVAL_ATTRS` the error slot is always present (0 when there is no error) and a
  folder's record holds no file attributes; the parser follows the returned mask, so both shapes parse. The next
  record starts `length` bytes on (the length includes the padding; every record was 8-byte aligned).
- Additions the step left open, taken from the design's section 3 and 01: a folder swapped for a link fails the open
  with `ENOTDIR`, not `ELOOP`, so `ENOTDIR` is `Replaced` when something still stands at the path and `Vanished`
  otherwise; a path past `PATH_MAX` (`ENAMETOOLONG`) is opened one name at a time with `openat` and `O_NOFOLLOW`; a
  folder that reads empty and whose handle has a link count of 0 was removed while read (`Vanished`).
- `read` clears the listing first. The device number is widened like `std`'s `st_dev` (sign-extended), so the reader,
  the root's identity and a portable reader agree. Other object types (FIFO, socket, device) are `File`, as in the
  reference walk. `head` opens the folder, then the file with `openat(O_RDONLY | O_NOFOLLOW | O_NONBLOCK)`, and refuses
  anything but a regular file as `Replaced`.
- Checks: 32 new tests (parser on made-up buffers, two proptests of any bytes and any masks, the reader on temporary
  folders: links, swaps, mode `000`, a FIFO, a clone, a folder of 10 000 entries against `std`, a 1 KiB buffer, a path
  past `PATH_MAX`); `cargo llvm-cov` on the crate: 97.2 % of lines. The numbers of a single-threaded read of the
  `small` tree wait for 05/01.

