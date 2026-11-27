# 043/03/01 — Turn them on

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** repo, sandbox, fixtures, docs

`high = offered` and `largest = shown`, in the release and development columns. Every Rust test is value-agnostic
since 01/02 and every e2e spec names its flags since 02/01, so the change is the file, one generated line and the
docs. The design: [`../00-design.md`](../00-design.md), section 8; agent's call 6.

## Read first

`flags.toml`; `crates/sandbox/src/expected.rs` (`WITHHELD_IN_THE_APP`, its tests) and how `EXPECTED.md` files are
written (`npm run sandbox -- <feature>`, `docs/testing.md`, "Sandboxes"); `docs/restrictions/deletion-safety.md`, "A
High plan needs its own tick" and "High is withheld in the first release"; `docs/architecture/README.md`, "The High
switch" and "The largest files"; `docs/changelog.md`, "Unreleased".

## Files

- `flags.toml` — `[high]` `release = "offered"`, `development = "offered"`; `[largest]` `release = "shown"`,
  `development = "shown"`.
- `crates/sandbox/src/expected.rs` — `WITHHELD_IN_THE_APP` becomes `SHOWN_WHILE_OFFERED`: "Shown in the app only while
  the build offers High (`flags.toml`, `high`)."; the comment at its use follows; the tests renamed with it.
- Every `EXPECTED.md` with the old line, regenerated (`npm run sandbox -- <feature>` per case file whose checklist
  holds a High positive: `rg -l "High is withheld in this release" fixtures`), never edited by hand.
- `docs/restrictions/deletion-safety.md` — "High is withheld in the first release" becomes "High is withheld when the
  build says so": the rule holds while the build's `high` flag is `withheld`; 0.1.0 withheld it, 0.2.0 offers it;
  the value is set only before the build (`flags.toml`, `FRESHER_FLAGS`), never by a file or setting the user can
  write. Why: the user's decisions of 2026-11-24 and 2026-11-26. "Enforced by" gains `app/tauri/build.rs` and
  `npm run check:flags`. "A High plan needs its own tick" loses "Unused while High is withheld".
- `docs/architecture/README.md` — "The High switch": set from `FLAGS.high`, offered in 0.2.0; "The largest files":
  shown in 0.2.0, its files removable at High.
- `docs/changelog.md` — "Unreleased": "Items at High are offered again, in their own group, each with its own tick in
  the confirmation"; "Largest is back in the sidebar: pick a large file to remove"; "What a build ships with is set in
  one place before the build, and the release's text lists it".

## Interfaces produced

- A 0.2.0 build that offers High and shows Largest, for 03/02's cases and 03/03's walk.

## Tests (negatives first)

- `crates/sandbox/src/expected.rs`: negative `a_high_positive_of_a_withheld_file_has_no_shown_while_offered_line`;
  positive `a_high_positive_says_when_it_is_shown_in_the_app`.
- The rest are existing tests run again: the compose and sandbox tests follow `FLAGS.high`; the harness still runs
  `high-withheld.toml` withheld (its proof that a withheld build offers nothing High stays).

## Checks

- `cargo nextest run -p fresher-sandbox -p fresher-app -p fresher-e2e`, `npm run sandbox:test`
- `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`
- `npm run check:flags`; `node scripts/release-version.ts notes 0.1.0 | tail -4` shows `high`: offered and
  `largest`: shown
- `npm run test:ui`; `npm run test:e2e:ui` (no baseline moves: every spec names its flags)
- `rg "High is withheld in this release|first release withholds" --glob '!docs/plans/**' .` finds nothing

Commit: `feat(app): offer High and show Largest in 0.2.0`.
