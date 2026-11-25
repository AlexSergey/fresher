# A small file's folder is opened through a link among its names

**Status:** open · **Updated:** 2026-11-25

## Symptom

A module reads a small text file its matches showed (a version pin, a pointer file) with `FolderReader::head`. The
macOS reader opens the file's folder with `O_NOFOLLOW`, which refuses a link only as the folder's own last name
(`crates/macos/src/bulk/reader.rs`, `head`). A folder above it that another program swapped for a link after the walk
listed it is followed, so a pin file outside the walked tree may be read. Found by the security audit of 2026-11-25
(SCAN-7).

## Cause

The walk reads each folder by its path and checks the folder's identity against its parent's listing; `head` has no
such identity check, and its folder path is opened by name like the walk's.

## What was tried

- Opening the folder with `O_NOFOLLOW_ANY` (no link in any name): it holds for the app, whose scan roots are
  canonical, but refuses any caller whose folder path goes through a link of the system (`/var` →
  `/private/var`), and the shell's tests read a small file of a temporary home by such a path. Left for a change
  that can update those callers too.

## How it is kept safe today

- `head` never follows a link as the file's own name, never blocks on a pipe, reads only a regular file and at most
  4 KiB (`crates/core/src/system/small_file.rs`).
- What a pin can do is keep a toolchain (`unused-toolchains`: an unknown or extra pin blocks or narrows offers), so a
  file read from elsewhere can only make the app keep more.

## Options left

- Open the folder with `O_NOFOLLOW_ANY` once every caller passes a canonical folder path (the scan's are).
- Or check the opened folder's identity against the one the walk listed, as a folder read does.
