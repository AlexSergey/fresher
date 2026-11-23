# 040 — The README

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** repo, scripts, ci, docs

Written on 2026-11-22 from the draft of the same day and a read of `main` at 33d51a6c. Approved by the user's
delegation of 2026-11-22 (see "Decisions"). The design, with its agent's calls: [`00-design.md`](./00-design.md).

## Intent

The user's task (translated): "Perhaps the README on GitHub should be updated too — and in general made better and
clearer, shorter, and by best practice. Make it a separate plan."

The README is the product's front door: most people decide in the first screen whether to download. Today's one
promises things that are not built (cleaning Docker, watch mode), has no picture of the app and no way to install it,
and spends its length on how the app is built rather than on what it does for the reader.

## Goal

A README about a third shorter, in the order: head, picture, install, what it finds, why you can trust it, what it
does not do, no warranty, help it get better, build and contribute, licence. Its "What it finds" block is generated
from `docs/categories/` (`npm run readme:generate`) and a pre-push and CI check (`npm run check:readme`) fails when the
README and the catalogue differ, a relative link or anchor is dead, an image is not in `assets/`, or the README claims
something not built (watch mode). Beside it, GitHub's community files in `.github/` (how to contribute, the security
policy, three issue templates), the repository's description and topics and a social image, for the user to apply.

Nothing in the app, the core or the UI changes. Branch: `feat/040-readme`, from `main` (33d51a6c), squash-merged into
`main` once when done ([`../../workflow.md`](../../workflow.md)). Plans 031, 032, 035, 036, 037, 038 and 039 merge into
`main` before this plan finishes; plan 04 merges `main` in and catches up with them.

## Plans

| #   | Plan                                      | Status      | Depends on | What                                                                              |
| --- | ----------------------------------------- | ----------- | ---------- | --------------------------------------------------------------------------------- |
| 01  | [The generator](./01-generator/readme.md) | 🔨 in progress | —          | the `**Example:**` line in every category file, the script, the link check        |
| 02  | [The text](./02-text/readme.md)           | 🔨 in progress | 01         | the new README, the stack table into `docs/`, the check in pre-push and CI        |
| 03  | [GitHub](./03-github/readme.md)           | 🔨 in progress | 02         | the community files, the issue templates, the social image, description and topics |
| 04  | [Finish](./04-finish/readme.md)           | 🔨 in progress | all, and plans 031–039 on `main` | catch up with `main` (examples, picture, wording), docs, conclusions |

### Order

```
01/01 ── 01/02 ── 01/03 ── 02/01 ──┬── 03/01 ──┬── 04/01 ── 04/02
                                   └── 03/02 ──┘
```

Each step's check is the previous step's output: the examples feed the script, the script feeds the README, the
README's links reach the files of 03. 03/02 can run beside 03/01 (it owns only `assets/social-preview.*`,
`docs/dependencies.md` and a section of `docs/plans/040-readme/03-github/02-social-and-settings.md`).

## Acceptance

- `npm run pre-push` passes on the branch's last commit; it includes `npm run check:readme`, and CI's `lint` job runs
  it too.
- `npm run test:scripts` passes, with `scripts/readme-catalogue.test.ts`, `scripts/readme-links.test.ts` and
  `scripts/readme.test.ts` (every refusal of the design's sections 2 and 3 as a negative, then the positives).
- Editing one cell of the generated table by hand, or one category file's danger level or example, makes
  `npm run check:readme` fail naming it (done once by hand per kind, written in 04/02).
- The README names no feature that is not on `main`, names no other product, and has no image outside `assets/` and
  shields.io.
- `git diff 33d51a6c -- crates modules app` is empty: no code of the app changes.

## Ideas and deferred items this touches

- [Best practices](../../ideas/best-practices.md), 27 (an always-current README): the table half of it; the picture
  comes with plan 038.
- New deferred item: The README at the first release (`docs/deferred/readme-at-release.md`), written in 04/02.

## Risks

- **A long table**: about 37 rows now, more after plans 031–037. Sorted by danger; folding it is one line (agent's
  call 11).
- **Prettier and the generator disagree**: the generator formats with Prettier's own API and config, and the check
  compares formatted text, so the pre-commit hook never rewrites the block.
- **Plans landing after this one** add a category file without `**Example:**`: the check fails their pre-push, and the
  catalogue's format tells them the line is needed.

## Agent's calls (summary)

The full list with costs is in [`00-design.md`](./00-design.md), "Agent's calls (to check)":

1. Only true claims (no watch mode; Docker never removed).
2. The pain first, the technology last.
3. No picture until plan 038's GIF exists.
4. A "What it does not do" block.
5. One ask for a star, at the end.
6. An explicit `**Example:**` line in every category file.
7. Images only from `assets/` or shields.io.
8. A list of claims not yet true in the check.
9. The repository's settings are the user's to apply.
10. GitHub's community files in `.github/`, an exception recorded in `restrictions/repository.md`.
11. The table sorted by danger, all rows visible.
12. English only.

## Open questions

None.

## Decisions

- 2026-11-22 — the user delegated: build autonomously, the recommended decisions and agent's calls taken.
- No version bump: 0.1.0.

## Conclusions

Built, awaiting the user's walk (the README on GitHub after a push, the "New issue" page, the social preview).

**What was built.** A README of 163 lines (124 before): 106 hand-written lines and the generated table (44 kinds, all
rows visible, sorted Safe, Low, Medium, High, then catalogue order). `npm run readme:generate` writes the table from
`docs/categories/`; `npm run check:readme` (in `pre-push` and in CI's `lint` job) fails on a table that differs, a dead
link or anchor, an image outside `assets/` or a remote one, a missing file or image, and a claim of watch mode. The
`**Example:**` line is in every category file. In `.github/`: `CONTRIBUTING.md`, `SECURITY.md` and three issue forms
(`wrong-finding`, `missing-folder`, `bug-report`); `assets/social-preview.{svg,png}`. No code of the app changed:
`git log --oneline main..HEAD -- crates modules app` shows nothing but the merge of `main`.

**What changed from the design.** (1) The README is not shorter by `wc -l`: the hand-written text is 106 lines and the
generated table adds 57; "shorter" holds for the prose, the table is the price of an always-current list (agent's call
11 keeps it visible). (2) No picture: plan 038 committed no `assets/demo/*.gif` (the recording is made by hand).
(3) The three `core-*` categories (the user's own files, uninstall keeping data or everything) are rows of the table
and are counted in the 44 kinds, though the user picks them rather than a scan finding them. (4) `config.yml` keeps
`blank_issues_enabled: false` with the security link, over plan 039's `true`; `wrong-finding.yml` is plan 039's, since
the app's recovery help links to it. (5) The link check became case-exact after the final review. (6) On a catalogue
problem `generate` writes nothing and `check` skips the block comparison.

**The numbers.** README 124 lines before, 163 after (106 outside the generated block). 45 category files with an
example (44 in the table). Script tests: 64 in the three `readme*.test.ts` files (25 catalogue, 30 links, 9 file
system and Prettier); `npm run test:scripts` 195 at the catch-up.

**By-hand mutation checks** (02/01; each reverted, each failed as it should): a changed table cell gave `stale-block`;
`Low` to `Safe` in `dev-caches-npm.md` gave `danger-mismatch`; "watch mode" in the README gave `unbuilt-claim` at the
right line.

**Plans of 031 to 039 and 04/01.** `main` b9cafed1 was merged in: 031 was the base; 037, 032, 038, 035, 036 and 039 came
in with the security fixes for `url` and `cookie`. The README's Docker sentence is plan 032's ("shows what Docker takes
and the command; never runs it"), "Trash first" and "No warranty" follow plan 039's agreement lines and glossary.
