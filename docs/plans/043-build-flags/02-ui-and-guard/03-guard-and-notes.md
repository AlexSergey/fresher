# 043/02/03 — The guard and the release text

**Status:** ✔️ done · **Version:** 0.2.0 · **Date:** 2026-11-26 · **Modules:** scripts, ci, docs

`npm run check:flags` holds `flags.toml`'s shape and that every flag is read; the release's text ends with "This
build", the flags it was built with. The design: [`../00-design.md`](../00-design.md), section 7; agent's calls 5 and
7.

## Read first

`flags.toml`; `app/tauri/build/flags.rs` (the rules the guard repeats for the override); `scripts/check-texts.ts` and
its test (the shape of a guard: pure checks, a `main` that prints violations and sets the exit code);
`scripts/release-version.ts` (`releaseBody`, the `notes` mode) and its test; `package.json` (`check:*`, `pre-push`);
`.github/workflows/ci.yml` (the `lint` job), `.github/workflows/release.yml` ("Gather the release files");
`docs/quality.md`, layer 5; `docs/release.md`, "What a release holds".

## Files

- `scripts/flags.ts` (new) — `readFlags(text): Flag[] | Violation[]` (the strict reader of the design, section 7:
  comments, blank lines, `[name]`, `key = "string"`, `values = ["a", "b"]`; any other line is a violation naming its
  number); `checkFlags(flags, sources): Violation[]` (the five keys and no other, `about` one sentence ending in a full
  stop and at most 120 characters, at least two kebab-case `values` with none repeated, `release` and `development`
  among them, each flag read as `flags.<name>` or `FLAGS.<name>` in `sources`); `resolveRelease(flags, overrides):
  …` (the build script's override rules); `notesSection(resolved): string` ("## This build" and one line per flag).
  `main`: `check` reads the file and the sources (`app/tauri/src/**/*.rs`, `app/ui/src/**/*.{ts,tsx}` without tests,
  stories and `*.generated.ts`); `buildSection` builds the section from the release column and `FRESHER_FLAGS`, and `release-version.ts notes` prints it.
- `scripts/flags.test.ts` (new).
- `scripts/release-version.ts` — `notes` appends `notesSection(..)` after the changelog section (`releaseBody` takes
  it as a fourth argument); its usage line and module comment follow. `scripts/release-version.test.ts` follows.
- `package.json` — `"check:flags": "node scripts/flags.ts check"`; in `pre-push` after `check:texts`.
- `.github/workflows/ci.yml` — `npm run check:flags` in the `lint` job after the text check. `release.yml` unchanged
  (its notes step runs `release-version.ts notes`; it sets no `FRESHER_FLAGS`).
- `docs/quality.md` — layer 5: `npm run check:flags` and what it holds; the CI table's `lint` row.
- `docs/release.md` — "What a release holds": the text ends with "This build", the flags of `flags.toml`'s release
  column; "Cutting a release": step 1 checks `flags.toml`'s release column is what the release must ship, and that a
  tag never builds with `FRESHER_FLAGS` (a local build or the dry run may).

## Interfaces produced

- `npm run check:flags`; the "This build" section in `release-version.ts notes`.

## Tests (negatives first)

`scripts/flags.test.ts`, `describe('negative cases')`:
- `refuses a line the reader does not know` (a TOML inline table, a number value)
- `refuses a table missing a key or holding another`
- `refuses an about that is empty, has no full stop or is too long`
- `refuses fewer than two values, a repeated one or one not in kebab-case`
- `refuses a release or development value outside its values`
- `refuses a flag no source reads` (only in a test file, a story or `bindings.generated.ts` counts as not read)
- `refuses an override naming an unknown flag, a value outside its values, a flag twice or no equals sign`

`describe('positive cases')`:
- `accepts the repository's flags.toml against the repository's sources`
- `takes the release column when no override is set`
- `an override replaces only the flags it names`
- `prints one line per flag in the file's order`

`scripts/release-version.test.ts`: positive `the body ends with the build's flags`.

## Checks

- `node --test scripts/flags.test.ts scripts/release-version.test.ts`
- `npm run check:flags`; `node scripts/release-version.ts notes 0.1.0` ends with "This build"
- `npm run lint:ts`, `npm run lint:code`, `npm run knip`
- `npx actionlint` is run by CI; locally `actionlint .github/workflows/ci.yml` if installed.

Commit: `feat(scripts): check:flags, and the release text lists the build's flags`.
