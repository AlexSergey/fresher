# 008/01/02 — Links, mounts and open files

**Status:** ✔️ done · **Version:** 0.5.0 · **Date:** 2026-11-16 · **Modules:** macos (probe)

## Files

- `NO_COMMIT/probes/008/guard/` — a Cargo project outside the workspace (`libc` only):
  - `nofollow` — opens paths with `O_NOFOLLOW_ANY | O_RDONLY` (and `O_DIRECTORY` for a folder): a plain file, a folder,
    a link as the last component, a link in the middle, `/var/…` and `/tmp/…` (links of the system), a firmlinked
    path; prints the result and `errno`;
  - `flags` — reads `st_flags` (user-immutable, system-immutable, restricted), `st_uid` and the mode through `std`'s
    `MetadataExt` on a locked file, a file of the system, a file of the user;
  - `mount-inside` — mounts a disk image at a folder inside a temporary folder (`hdiutil attach -mountpoint`), then
    deletes the outer folder's copy (a second, throwaway tree) with `std::fs::remove_dir_all` and reports whether it
    descended into the mounted volume; then the same with the probe's own walk that refuses a different device;
  - `open-files` — lists the user's processes (`proc_listallpids`), each one's open descriptors
    (`PROC_PIDLISTFDS`), their paths (`PROC_PIDFDVNODEPATHINFO`), its working folder (`PROC_PIDVNODEPATHINFO`) and its
    name; counts processes, descriptors, failures by `errno`, and the time; checks it sees a file the probe holds open
    in a child process and a browser's cache file while the browser runs.

## Questions to answer

1. Is `O_NOFOLLOW_ANY` available on macOS 13 and later, and in `libc` 0.2.191? It fails with which `errno` for a link
   in the middle and at the end? Can a folder be opened with it for `fstat`? Can a link that is itself the item be
   opened as the link (`O_SYMLINK`) while every other component still may not be a link?
2. Temporary folders: the canonical form of a `tempfile` root (`/private/var/folders/…`), so tests open beneath it.
3. Are the three file flags and the owner readable without opening the file?
4. Does `remove_dir_all` cross into a volume mounted inside the folder? (Expected yes: then the folder walk's device
   check is what protects a permanent delete.)
5. The open-files snapshot: how many processes and descriptors on the developer's Mac, how long one snapshot takes,
   whether processes of other users answer `EPERM` (and nothing else), whether memory-mapped files without a
   descriptor are missed (a database a browser maps), and whether the process name is enough for a plain sentence
   ("Quit Safari and try again").
6. Does reading another process's descriptors raise any prompt, with or without Full Disk Access?

## Produces

The probe's `RESULTS.md`, read by step 03: the flags and constants plan 03 and 04 use, the snapshot's cost (does a
refresh every 2 s hold?), the limits written in the threat model.

## Checks

- Every question has an answer or "could not tell, because …".
- The probe deletes only its own throwaway trees; the mounted image is detached at the end.
- No output contains a name or path from the home folder or of another process's files (counts only, except the
  probe's own test files).

## Conclusions

Done on 2026-11-16 (Mac15,7, macOS 27.2.1); the numbers are in the design, "Measured (01)", items 1–6 of links, mounts
and open files. The probe deleted only its own trees and detached its image.

- `O_NOFOLLOW_ANY` (0x20000000 in `libc` 0.2.191) refuses a link anywhere with `ELOOP`; with `O_SYMLINK` it opens a
  last-component link as the link and still refuses a middle one, so 04/02 needs no fallback. Roots must be
  canonical: `/var/folders/…` and `/tmp` fail, `/private/…` opens; firmlinks pass.
- File flags, owner and mode come from `symlink_metadata` without opening.
- `remove_dir_all` empties a volume mounted inside the folder before failing `EBUSY`: the device check of the folder
  walk (call 18) is what protects a permanent delete.
- The open-files snapshot takes 7–19 ms (a 2 s refresh holds), raises no prompt, and other users' processes answer
  `EPERM` only. It misses files mapped without a descriptor (348 in the home folder): the macOS adapter adds a region
  walk (0.4–0.9 s) reused for 30 s — design, call 38. Helper processes need the app bundle's name (call 35).
- **Needs the user**: a run with Full Disk Access; macOS versions before 27.
