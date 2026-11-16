# 009/03/02 — The team id reader

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** system

## Files

- `crates/system/src/signature.rs` (new) — `team_id(file: &mut (impl Read + Seek)) -> Option<String>`: the Mach-O
  header (64- and 32-bit, little-endian; universal files: the first slice), the `LC_CODE_SIGNATURE` command, the super
  blob, the code directory, its team identifier from version 0x20200. Bounded reads: 64 KiB of load commands, 1 MiB
  of signature.

## Checks

- Negative (first): not a Mach-O; no signature command; a signature outside the file; a code directory older than
  0x20200; a team offset past the blob; property tests: no input panics.
- Positive: a thin 64-bit file, a universal file and a code directory after other blobs give the team id.

## Conclusions

- Built as written by a subagent in a scratch crate, then moved into `crates/system/src/signature.rs` (tests in
  `signature/tests.rs`, with a test-only Mach-O builder); committed with 03/03.
- Only the primary code directory (slot 0) is read; a universal file tries at most eight slices, the first that
  gives a team id wins; more than 32 slices is not a universal file (the Java class magic); big-endian thin files
  answer `None`. A signature over 1 MiB is read in its first MiB. The answer is 1–32 visible ASCII characters; the
  caller keeps it only as a valid `TeamId`.
- Checked by hand, not kept: five third-party apps gave the team ids the system's own signing tool reports; `/bin/ls`
  (an Apple platform binary, three slices) gives none. A test that reads `/bin/ls` and a system app is `#[ignore]`d.
  35 tests (22 negative, 8 positive, 4 property, 1 ignored).
