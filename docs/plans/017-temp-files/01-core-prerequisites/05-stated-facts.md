# 017/01/05 — The sandbox's stated facts

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-17 · **Modules:**
testing, sandbox, app, docs

Design section 6 and decision 18. A status-change time cannot be set by any tool, owners and system flags need root,
and a socket's path is limited to 104 bytes: a case states these facts, and a lens over the real reader reports them
for exactly the stated paths.

## Files

- `crates/testing/src/tree_description.rs` — `[[file]]` and `[[dir]]` gain `changed`, `accessed` (`"<n>d"`, `"<n>h"`,
  or `"<n>s"`, counted back from the build's "now"), `owner` (`"system"`, `"other-user"`), `flags` (`"user-lock"`,
  `"system-lock"`, `"no-unlink"`, `"restricted"`); `[[file]]` gains `kind` (`"socket"`, `"pipe"`); `age` may be
  negative (`"-2d"`: a modification time after now). A stated fact on a path the tree does not write is refused.
- `crates/testing/src/facts_lens.rs` (new), `lib.rs` — `StatedFacts` (path → the stated values, canonical paths) and
  `FactsLens<R: FolderReader>`: answers every call of the wrapped reader, then replaces, for a listed entry only, its
  `changed`, `accessed`, owner (`0` for system, `502` for another user), flags and kind. Never a dependency of a
  release build (a dev-dependency, and the sandbox composition's feature).
- `crates/testing/src/{fixture_tree,run_module}.rs` — `FixtureTree` keeps its tree's `StatedFacts`; `run_module`
  wraps the scanner's reader with the lens when there are any.
- `crates/sandbox/src/{layout,generate}.rs` — the sandbox's layout gains `private/var/folders/xx/sandbox/T/` (with
  `C/`, `0/`, `X/` beside it, empty) and `private/tmp/`; its known folders `UserTemp` and `SharedTemp` point there;
  its protection list mirrors 01/01's on them. A case's paths may start with `private/`.
- `crates/sandbox/src/world.rs` — `world.toml` gains `[[entry]]` (a sandbox path and its stated facts), joined over the
  case files; `World::stated()` gives `StatedFacts`.
- `crates/sandbox/src/expected.rs` — a stated fact shows in `EXPECTED.md` ("stated: owned by the system, changed 30
  days ago"), since a run by hand cannot see it in Finder.
- `app/tauri/src/sandbox.rs` — sandbox mode wraps the real reader with the world's lens; `sandbox_engine` takes it;
  `app/tauri/tests/sandboxes/harness.rs` the same.
- `docs/testing.md` ("Sandboxes", "The case file") — the stated facts, the lens, the two temporary places; the sentence
  that a lens fact is never what Finder shows.

## Produces

Fixture homes and case files that state old status changes, owners, flags, sockets and pipes, for 02/03, 03/01 and
03/02.

## Checks

- Negative first: a stated fact on a path not written, an unknown owner or flag, `kind` on a `[[dir]]`, a `kind` with
  `fill` are refused; an entry not listed is reported exactly as the real reader reads it (a property test over random
  trees: the lens with no facts equals the reader); the lens never answers for a path outside the tree.
- Positive: a stated `changed = "30d"` file reads 30 days old in `Match.changed` and in its folder's `Inside`; a stated
  socket is `Other`; a stated system owner makes `owned_by_user` false and `Inside.foreign` true; `world.toml` round
  trips its entries; `all` joins two case files' entries; sandbox mode's reader reports a stated fact.
- `cargo nextest run -p fresher-testing -p fresher-sandbox -p fresher-app`; `npm run sandbox:test` (every existing
  golden unchanged).

## Conclusions

Built (8c4393b): the tree vocabulary's stated facts (`changed`, `accessed`, `owner`, `flags`, `kind`, ages in days,
hours or seconds, negative for after now; `kind` with an explicit `fill` refused), `FactsLens` and `StatedFacts` in
`fresher-testing`, `FixtureTree::stated`, the sandbox's two places with their own protection
(`Sandbox::places_protection`, joined with the Mac's list, which is never given the sandbox's `T`), `world.toml`'s
`[[entry]]` lines joined over the case files, the stated lines of `EXPECTED.md`, and sandbox mode and the harness
reading through the lens.

Changed from the brief:

- `run_module` walks with `NaiveWalk`, which applies the same facts itself (`with_stated`): `fresher-testing` cannot
  depend on the scanner. The lens is used by the scanner's tests, sandbox mode, the harness and the core e2e.
- A stated fact is a field of the entry it states, so it is always on a written path; the refusal of a path the sandbox
  does not hold lives in `World::read`.
- The harness's tree after leaves out the ten empty layout folders, so every earlier golden stayed unchanged; a sandbox
  generated before this step must be generated again.
- `other-user` read as uid 502, which could be the user running the tests; 02/03 changed it to 4294967294 (`nobody`).
- A file written now has a status change after the fixtures' `FIXTURE_NOW`: a case meant to look old states `changed` on
  every entry inside it (written in `docs/testing.md`).

The sandbox-mode test with a stated system owner and lock fails with the lens emptied (checked by mutation).
