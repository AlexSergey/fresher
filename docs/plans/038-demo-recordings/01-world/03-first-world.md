# 038/01/03 — The first world and its scan test

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** fixtures, sandbox, app

The first demo world, `fixtures/demos/developer-mac.toml`: a believable developer's Mac on which every headline module
has something to show at several danger levels, and a test that scans every demo world in sandbox mode with every
module and fails when a category of its `shows` offers nothing. The design: [`../00-design.md`](../00-design.md),
section 1.

Read first: 01/01 and 01/02 (as built); every `fixtures/sandboxes/*.toml` whose module the world uses (their positive
cases show the exact shapes each module claims: paths, apps in `[[case.app]]`, ages, receipts, access);
`app/tauri/tests/sandboxes.rs` and `app/tauri/tests/sandboxes/harness.rs` (how a sandbox is generated, opened and
scanned with `sandbox_engine` and fake ports); `docs/restrictions/repository.md` ("Clean room").

## Files

- `fixtures/demos/developer-mac.toml` (new) — `feature = "developer-mac"`, no `module`; a head comment: what a demo
  world is, that nothing in it is checked but `shows`, and that every name is made up. `[demo]`:
  `now = 1795000000`, `sizes = "as-declared"`, `[demo.volume]` `name = "Macintosh HD"`, `capacity = 494384795648`
  (460 GiB), `used` and `free` such that `used` exceeds the world's declared total by about 300 GB and
  `used + free = capacity`. `access` stated as granted for every guarded folder the modules read (so no permission card
  shows), and `macos = "15.4"` in one case where a module needs it. Scenery cases, one per story, each with realistic
  made-up names and sizes (sparse files; the sizes below are the targets, within ±20 %):
  - Xcode's build products of three made-up projects (`dev-caches.xcode-derived-data`, ~38 GB in all);
  - npm, Homebrew, Cargo and Gradle caches (`dev-caches.*`, 2–9 GB each);
  - the caches of three installed made-up apps in `~/Library/Caches/<bundle id>` and two browsers' caches
    (`app-caches.*`, 0.5–4 GB each; the apps in `Applications/` via `[[case.app]]`, not running);
  - the leftovers of two removed apps, one with data at High (`app-leftovers.*`);
  - two local AI models at Medium (`local-ai-models.*`, 4–8 GB each);
  - an old Node version and an old Rust toolchain (`unused-toolchains.*`);
  - old installers and a big disk image in `Downloads` (`downloads.*`), logs (`logs.*`), a stale project
    (`stale-projects.*`) and temporary files (`temp.*`) — each only when the module's own case file shows the shape
    needs nothing the world cannot state; a module left out is named in the head comment with why.
  - The user's own files that are never offered (Documents, Photos library-like folders, a code folder), several GB,
    so the storage map's "your folders" is believable.
  `shows` lists exactly the category ids that the scan must offer (every category a scenery case targets).
  No real person's or company's name; app bundle ids under `com.example.` or made-up vendors; no product of another
  disk cleaner.
- `app/tauri/tests/demos.rs` (new, `#![cfg(debug_assertions)]`, helpers under `#[cfg(test)]` as `sandboxes.rs`) — one
  `#[test]` per demo world via a `demo_worlds!` macro listing each file stem, and a guard test that every
  `fixtures/demos/*.toml` has a line (as `sandboxes.rs` guards its case files). Each test: parse the world, generate it
  into a temporary folder at its `now`, compose the app's sandbox engine with every module (the harness's way, with its
  fake ports; the world's stated facts and volume as 01/02 reads them), scan, and assert: the scan finishes; every id
  of `shows` has at least one finding; no finding's path is outside the sandbox; the storage map's volume capacity is
  the world's. A failure names the world and the category. Reuse `harness.rs` helpers by `#[path]` where they fit
  rather than copying them; a helper made `pub(crate)` for this is fine.
- `crates/sandbox` — a unit test (in `main.rs`'s or `case.rs`'s tests) that every `fixtures/demos/*.toml` parses as a
  demo world.

## Produces

The world `developer-mac` for the scenarios of 02/01 and 04/01, and a CI guard that it keeps showing what it should.

## Checks

- `cargo nextest run -p fresher-app --test demos` passes; changing one `shows` id to a category the world does not
  feed fails naming it (done once by hand, then reverted; written in the step's conclusions).
- `cargo run -q -p fresher-sandbox --locked -- generate fixtures/demos/developer-mac.toml --into
  /private/tmp/fresher-demo/try` writes the world (then remove that folder by hand); `du -sh` of it is small (sparse
  files), and its `world.toml` lists `allocated` entries and a `[volume]`.
- The world's file is under the repository's size limit; `typos` passes on it.
- `cargo nextest run -p fresher-sandbox -p fresher-app`; clippy; fmt.

## Conclusions

Built as briefed: `fixtures/demos/developer-mac.toml` (11 scenery cases, 58 files, declared size about 144 GB, a
stated 460 GiB volume with 444 GB used, `as-declared` sizes, 21 `shows`) and `app/tauri/tests/demos.rs`, which scans
every world of `fixtures/demos/` and names the world and the category that offers nothing.

- xcode, docker, trash, jetbrains and Ollama are left out of `shows`, each for a reason in the world's head comment;
  the Xcode story is the `dev-caches` derived data.
- The used space of the volume exceeds what the world declares by about 300 GB: the rest of the made-up disk.
- A second world per story is a new file; the guard `every_demo_world_has_a_test` asks for its test line.
