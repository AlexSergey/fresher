# 007/01/03 — The generator

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-16 · **Modules:** sandbox, repo

## Files

- `crates/sandbox/src/layout.rs` — `Sandbox`: the folder's parts (the marker, `home/`, `home/.Trash/`,
  `Applications/`, `System/Applications/`, `.fresher-data/`) and its known folders (`FakeKnownFolders::home_layout`
  of `home/`, with the two apps folders set). `Sandbox::open(path, user)` checks the marker by the cleaner's rules (a
  regular file of the user, opened through no link, the signature on its first line) and the layout.
- `crates/sandbox/src/generate.rs` — `generate(files, into, now, real_protection, user) -> Generated { sandbox, strangers }`: refuses a non-empty folder without
  a valid marker, a folder the real Mac's protection list refuses (never, an ancestor, a whole folder: so never the
  real home itself) and a path through a link; writes the marker (the signature, then a line naming the case files),
  the layout and every case's tree. On a folder it made before it tops up — writes what is missing, resets the sizes
  and times of its own files — and returns the entries it did not make (`strangers`) with the sandbox. `user` is the id the marker must belong to
  (the binary passes the real user's). It removes nothing (decision 4).
- `crates/sandbox/src/expected.rs` — `EXPECTED.md`: per case, negatives first, a checkbox and one plain sentence
  ("never offered: …; still there after the cleanup", "offered in *Trial caches* (Safe): …; in the sandbox's Trash
  after the cleanup"); then the command that opens the app on the sandbox.
  Built: the checklist names the category by its id (the crate cannot read module texts); titles come with the
  sandbox run in the app.
- `crates/sandbox/src/main.rs` — `fresher-sandbox generate <feature|all> [--into <dir>]`: reads
  `fixtures/sandboxes/<feature>.toml` (every file for `all`), writes into `sandbox/<feature>/` at the repository's
  root by default; the real protection list comes from `fresher-macos` for the real home (the binary's one home
  lookup); prints what it wrote and every entry it did not make. Arguments are read by hand: no new dependency.
- `package.json` — `"sandbox": "cargo run -q -p fresher-sandbox --locked -- generate"`. `.gitignore` — `/sandbox/`.

## Produces

The sandbox folder that plan 02 opens and plan 03 tests.

## Checks

- Negative: `a_non_empty_folder_without_a_marker_is_refused`, `a_folder_with_an_unsigned_marker_is_refused`,
  `a_protected_folder_is_refused` (a protection list that makes the target whole), `a_target_through_a_link_is_refused`,
  `a_second_run_removes_nothing` (a stranger file stays and is listed; a changed file of its own is reset),
  `all_refuses_two_features_writing_one_path`.
- Positive: `a_case_file_becomes_its_sandbox` (the `TreeReport` of the result),
  `the_checklist_lists_every_case_negatives_first`, `a_generated_sandbox_opens`.
- By hand, once plan 03 adds `trial.toml`: `npm run sandbox -- trial` writes `sandbox/trial/`; `git status` stays
  clean.

## Conclusions

`generate` writes a feature's sandbox folder with `EXPECTED.md` and refuses a folder that holds strangers. Deviations: it
takes an extra `user: u32` (the marker's owner is checked as the cleaner does), and `EXPECTED.md` names categories by id.
Review findings: a file with more than one link is never rewritten; a protected place inside the sandbox is refused; the
marker check, duplicated from the cleaner, was moved to `fresher-macos` in 02/01.
