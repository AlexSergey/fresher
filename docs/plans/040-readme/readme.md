# 040 — The README

**Status:** 📝 draft · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** repo, scripts, docs

A one-file draft: the design in short, with the recommended decisions taken as agent's calls. Plans, steps and
`00-design.md` are written when the draft is approved.

## Intent

The user's task (translated): "Perhaps the README on GitHub should be updated too — and in general made better and
clearer, shorter, and by best practice. Make it a separate plan."

The README is the product's front door: most people decide in the first screen whether to download. Today's one
promises things that are not built (cleaning Docker, watch mode), has no picture of the app and no way to install it,
and spends its length on how the app is built rather than on what it does for the reader.

## Goals

1. In the first screen a visitor sees what the app is, what it looks like, and how to get it.
2. Every claim is true of the released build; a claim that stops being true fails a check.
3. Shorter: the README answers "should I try it"; everything else is one link away.
4. A developer sees their own disk in it: the folders and tools they know, with the danger levels.

## Non-goals

- A website, guides for search engines, the launch posts: promotion is planned apart.
- Naming or comparing with other products (the clean room,
  [`restrictions/repository.md`](../../restrictions/repository.md)).
- Translated READMEs before the interface is translated.

## Design

**The order**, top to bottom, about a third shorter than today:

| Block             | What                                                                                              |
| ----------------- | ------------------------------------------------------------------------------------------------- |
| Head              | logo, the one-sentence pain ("Your Mac is full. See why."), the developer line under it, few badges |
| Picture           | the `overview` animation of plan [038](../038-demo-recordings/readme.md), under the head            |
| Install           | the download link to the latest release, the package manager's one line, the macOS version, the first-launch note |
| What it finds     | a table: what, example, danger level — generated from [`categories/`](../../categories/README.md) |
| Why you can trust it | four lines, each linking to the code or the doc that enforces it: confirms first, Trash first and put back, nothing leaves the Mac, nothing in the background |
| What it does not do | one honest paragraph: no antivirus, no speed-up, no background helper, Docker is described but not cleaned, the Trash is never emptied for you |
| No warranty       | the agreement's words from `core.agreement.*` (plan [039](../039-agreement-and-wording/readme.md)) |
| Help it get better | "the app offered something it should not have", "a folder it should know" — the issue templates; a one-line ask for a star |
| Build and contribute | three commands and a link to the docs                                                          |
| Licence           | one line                                                                                          |

- **What leaves the README**: "How it is built" and the long build notes move behind a link to
  [`docs/`](../../README.md); the levels' table is folded into "What it finds".
- **Badges**: the release, the licence, the macOS version, "no network"; the status badge goes when 0.1.0 ships.
- **Truth by construction**: the "What it finds" table sits between two markers and is written by a script from the
  category files (`npm run readme:generate`); a check in pre-push fails when the README and the catalogue differ, when
  a link in it is dead, or when it names an image that is not in `assets/`.
- **The voice** follows the writing guide of plan 039: plain, calm, no superlatives.
- **Beside the README**: the repository's description and topics, the social preview image, and the files GitHub looks
  for — how to contribute, how to report a security problem, the issue templates. They are GitHub's own configuration
  under `.github/`; their text is short and points into `docs/`
  ([CLAUDE.md](../../../CLAUDE.md), "Documentation": nothing outside `docs/` but code and its configs).
- **Before the release exists**, the Install block says so in one line and shows the build commands; the plan's last
  step swaps it when plan [013](../013-release/readme.md) publishes.

**Testing.** The generator and the check are scripts with tests, negatives first: a category with no file, a table
edited by hand, a dead relative link, a missing image; then a README in step with the catalogue. No sandbox: nothing
here touches a disk. The finish is a look at the rendered page on GitHub, in the light and the dark theme and on a
phone's width.

## Plans (after approval)

| #   | Plan      | What                                                                 |
| --- | --------- | -------------------------------------------------------------------- |
| 01  | Generator | the table from the categories, the check, pre-push                   |
| 02  | The text  | the new order and wording, what moves into `docs/`                   |
| 03  | GitHub    | description, topics, the social image, the contribution and security files, templates |
| 04  | Release   | the picture from plan 038, the Install block from plan 013, the look |

01–03 can be done now. 04 waits for plans 013 and 038; the wording waits for plan 039's guide if that comes first.

## Ideas this touches

[Best practices](../../ideas/best-practices.md), 27 (an always-current README).

## Open questions

None.

## Decisions

None yet.

## Agent's calls (to check)

1. **Only true claims**: watch mode leaves the README until plan [010](../010-history/readme.md) is built; Docker is
   described as "explained, not cleaned".
2. **The pain first, the technology last**: Rust and the architecture move down or out.
3. **The table is generated**, never written by hand.
4. **A "What it does not do" block**: honesty is the positioning.
5. **One short ask for a star**, at the end, not at the top.
6. **English only** for now.

## Conclusions

Not written yet.
