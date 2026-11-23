# 040/02/01 — The new README and its check

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** repo, scripts, ci, docs

Rewrite `README.md` in the design's order, shorter, with only true claims; put the generator's markers in it and run
it; move the stack table into `docs/README.md`; wire `npm run check:readme` into `pre-push` and CI. The design:
[`../00-design.md`](../00-design.md), section 1.

## Context

- Today's `README.md` (read it first) promises watch mode (not built: plan 010 is a draft) and cleaning Docker images
  and volumes (not built; plan 032 will describe Docker and never clean it). Both go.
- 01/02 and 01/03 made `npm run readme:generate` and `npm run check:readme` (`scripts/readme.ts`): the block between
  `<!-- what-it-finds:start … -->` and `<!-- what-it-finds:end -->` is written from `docs/categories/`; the check also
  refuses dead relative links and anchors, images outside `assets/`, remote images other than `img.shields.io`, and the
  words "watch mode". Copy the start marker exactly from `START_MARKER` in `scripts/readme-catalogue.ts`.
- Facts: lowest macOS 13.0 (`app/tauri/tauri.conf.json`); Node `>=24.15.0`; version 0.1.0, no release yet; the bundle
  is built by `npm run build` (`tauri build`) — confirm the output folder by reading `app/tauri/tauri.conf.json` and
  the Tauri docs' default (`target/release/bundle/macos/` under the workspace's target folder); do not run the build
  if it is not needed to confirm.
- `.github/CONTRIBUTING.md` and the issue templates do not exist yet (03/01 writes them and adds their links here). Do
  not link them in this step: a dead relative link fails the check.
- The voice: plain, calm, second person, no superlatives, no exclamation marks, no emoji. Never name another disk
  cleaner or reference project. English only.

## Files

- `README.md` — replaced. Its blocks, in order (headings are `##`; the anchors in brackets must resolve):

  1. **Head** (centred HTML, as today): `assets/logo.png` at 160 × 160 with its alt text; `<h1>Fresher</h1>`;
     `<b>Your Mac is full. See why.</b>`; the line "A free, open-source disk cleaner for macOS that explains every
     gigabyte in plain English, shows the risk of removing it, and deletes nothing you have not confirmed. Made with
     developers' Macs in mind."; four badges:
     - `https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-3a3a3a?style=flat-square` (alt "Licence: MIT OR
       Apache-2.0"),
     - `https://img.shields.io/badge/macOS-13%2B-1c1d21?style=flat-square&logo=apple` (alt "macOS 13 or later"),
     - `https://img.shields.io/badge/network-none-2ea44f?style=flat-square` (alt "No network access"),
     - `https://img.shields.io/badge/status-pre--release-d69e2e?style=flat-square` (alt "Status: pre-release");
     the anchors' line: Install (`#install`) · What it finds (`#what-it-finds`) · Why you can trust it
     (`#why-you-can-trust-it`) · Build and contribute (`#build-and-contribute`).
  2. **Picture**: none in this step (plan 038's GIF is added in 04/01). Leave no placeholder.
  3. `## Install` — "There is no release yet. Until there is, you can build Fresher yourself on macOS 13 or later, with
     [Rust](https://rustup.rs) and Node.js 24:", then a `bash` block: `git clone
     https://github.com/AlexSergey/fresher.git`, `cd fresher`, `npm install`, `npm run build`; then "The app is written
     to `<the bundle folder>`. A build you make yourself is not signed: the first time, open it with right-click →
     Open."
  4. `## What it finds` — one sentence: "Each kind of file Fresher can offer has a page saying what it is, what
     happens if you remove it, and how the app decides." Then the two markers, nothing between them (the generator
     fills it).
  5. `## Why you can trust it` — four bullets, each a bold lead and one sentence with its link:
     - **It asks first.** Nothing is removed until you confirm the exact items, with their size and danger level
       ([how](docs/restrictions/deletion-safety.md#explicit-confirmation-only)).
     - **Trash first, and you can put it back.** Items go to the Trash, and Cleanups puts them back; deleting for
       good is a separate choice you tick each time ([how](docs/restrictions/deletion-safety.md#trash-first)).
     - **Nothing leaves your Mac.** No network requests at all: no telemetry, no crash reports, no update checks
       ([how](docs/restrictions/privacy.md#no-network)).
     - **Nothing in the background.** No helper, no login item, no scheduled scans: it works only while it is open
       ([why](docs/goal.md#8-transparent-it-does-only-what-the-user-asks)).
  6. `## What it does not do` — one paragraph: "Fresher is not an antivirus and does not "speed up" your Mac or clean
     its memory. It has no background helper. It never removes what it cannot name: space it does not recognize is
     shown as its real folders. It never removes anything of Docker's. And it never empties the Trash for you: you get
     the space back when you empty it yourself."
  7. `## No warranty` — "Fresher is free and open source and comes with no warranty. Before you use it, know that:"
     and five bullets: "It removes only what you confirm, so each removal is your decision."; "A danger level is the
     app's best judgement, not a guarantee."; "Keep a backup of anything you cannot lose."; "Removed items go to the
     Trash and can be put back until you empty it."; "The full terms are in the licences below."
  8. `## Help it get better` — "The most useful report is a wrong finding: something the app offered that it should
     not have. Open an issue with the path it showed (write your home folder as `~`) and what the folder really is. A
     folder Fresher should know about is welcome too." Then: "If Fresher helped you, a star helps others find it."
  9. `## Build and contribute` — a `bash` block: `npm install`, `npm run dev`, `npm test`; then "The goal, the rules
     and the architecture are in [`docs/`](docs/README.md); the checks every change passes are in
     [`docs/quality.md`](docs/quality.md). Fresher is a Rust core with a Tauri 2 shell and a React and TypeScript
     interface."
  10. `## Licence` — "Free and open source, under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
      Notices: [NOTICE](NOTICE). Created by [Serhii Aleksandrov](https://github.com/AlexSergey) and the Fresher
      contributors." (the author line kept from today's README).

  Then run `npm run readme:generate`, which fills the block and formats the whole file with Prettier.

- `docs/README.md` — a section `## The stack` after the table: today's "How it is built" table of the README (Core,
  Shell, Interface, Quality), unchanged, and one line "Build commands: [`quality.md`](./quality.md) and
  [`testing.md`](./testing.md)."
- `package.json` — `pre-push` runs `npm run check:readme` right after `npm run lint` (before `check:crate-deps`).
- `.github/workflows/ci.yml` — job `lint`: a step `- run: npm run check:readme` after `- run: npm run lint`.
- `docs/quality.md` — "Layer 5": the pre-push list names `npm run check:readme` ("the README's generated table, links,
  images and claims; `npm run readme:generate` rewrites the table"), and the CI table's `lint` row names it.

## Checks

- `npm run check:readme` passes; `npm run readme:generate` then `git diff --exit-code README.md` (the generator is
  stable on its own output).
- `npx prettier --check README.md` passes (the hook will not reformat it).
- `grep -i -n 'watch mode\|docker images\|docker data is removed' README.md` prints nothing.
- The README's prose (the lines outside the generated block) is shorter than the old README, and every block of the design's section 1 but the picture is there.
- By hand once, then reverted: change one cell of the generated table → `check:readme` fails with `stale-block`;
  change `**Danger level:** Low` to `Safe` in `docs/categories/dev-caches-npm.md` → it fails with `danger-mismatch`;
  write "watch mode" in the README → `unbuilt-claim`. Record the three in this step's conclusions.
- `npm run test:scripts`; `npm run lint:ts`; `npx actionlint` is not required locally (CI runs it); keep the YAML's
  indentation.
- Commit: `docs(repo): a shorter README with a generated category table`.

## Conclusions

- `README.md` rewritten in the design's order; the generator filled the table (41 categories). `check:readme` passes,
  a second `readme:generate` changes nothing, and `prettier --check README.md` passes.
- The bundle folder, `target/release/bundle/macos/Fresher.app`, comes from the workspace (`app/tauri` is a member of
  the root `Cargo.toml`, so the target folder is the root's) and Tauri's default bundle path; the build was not run.
- Line count: 151 against 124 before. The hand-written text is about 100 lines; the generated table adds 55. The
  README is shorter in words, not in lines, because the table is new.
- Checked by hand, then reverted: one cell of the table changed → `check:readme` fails ("the "What it finds" block is
  out of date", `stale-block`); `**Danger level:** Low` → `Safe` in `dev-caches-npm.md` → it fails ("the file says
  Safe, the catalogue says Low", `danger-mismatch`); "watch mode" written in the README → it fails ("watch mode is not
  built (plan 010)", `unbuilt-claim`).
- Fixed from 01/03's review: `slugOf` keeps `_` as GitHub does (03-link-check.md updated); single-quoted `src` and
  `href` are read; `[top](#)` is accepted; problems are sorted by a numeric line kept beside each problem.
