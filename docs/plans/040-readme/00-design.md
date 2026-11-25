# 040 — The README: design

**Status:** 🔨 in progress (awaiting the user's walk) · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** repo, scripts, ci, docs

Written on 2026-11-22 from the draft and a read of `main` at 33d51a6c (plan 034). Approved by the user's delegation of
2026-11-22 (see "Decisions").

## Intent

The user's task (translated): "Perhaps the README on GitHub should be updated too — and in general made better and
clearer, shorter, and by best practice. Make it a separate plan."

The README is the product's front door: most people decide in the first screen whether to download. Today's one
promises things that are not built (cleaning Docker, watch mode), has no picture of the app and no way to install it,
and spends its length on how the app is built rather than on what it does for the reader.

## Goals

1. In the first screen a visitor sees what the app is, what it looks like (once plan 038 has made the picture), and
   how to get it.
2. Every claim is true of `main`; the claims that can be checked by a machine fail a check when they stop being true.
3. Shorter: the README answers "should I try it"; everything else is one link away.
4. A developer sees their own disk in it: the folders and tools they know, with the danger levels.

## Non-goals

- A website, guides for search engines, the launch posts: promotion is planned apart.
- Naming or comparing with other products (the clean room, [`restrictions/repository.md`](../../restrictions/repository.md)).
- Translated READMEs before the interface is translated.
- Changing the repository's settings on GitHub (description, topics, the social image, private vulnerability
  reporting): the plan writes the texts and the image; the user applies them (agent's call 9).
- The Install block of a released app: it waits for plan [013](../013-release/readme.md) (deferred, section 7).

## Facts read on 2026-11-22

| Fact                         | Value                                                                                         |
| ---------------------------- | --------------------------------------------------------------------------------------------- |
| README location              | `README.md` at the repository root, formatted by Prettier at commit (`.lintstagedrc.cjs`: `*.md`; `docs/` is in `.prettierignore`, the root README is not) |
| Lowest macOS                 | 13.0 (`app/tauri/tauri.conf.json`, `bundle.macOS.minimumSystemVersion`)                       |
| Node                         | `>=24.15.0` (`package.json` `engines`)                                                        |
| Version                      | 0.1.0, no release yet (plan 013 is a draft)                                                   |
| Categories                   | 37 files in `docs/categories/`, each with an H1 title and a `**Danger level:**` line; the catalogue table in `docs/categories/README.md` lists them (category id, danger, rule, file) |
| Who reads the danger line    | `crates/testing/src/contract.rs` (`catalogue_problem`): the first word after `**Danger level:**` only; another line in the file does not disturb it |
| `.github/`                   | `workflows/ci.yml`, `workflows/codeql.yml`, `actions/setup/`; no community files, no issue templates |
| GitHub's description today   | "Make Mac fresh again"; no topics; no homepage (`gh repo view`, read-only)                     |
| Repository scripts           | `scripts/*.ts`, run by Node 24 directly (`node scripts/x.ts`), tested with `node --test` (`npm run test:scripts`), kebab-case names, knip entry `scripts/*.ts` |
| Tools on the developer's Mac | `rsvg-convert` 2.62.2 (librsvg), ImageMagick 7.1.1, `ffmpeg`, `gh` 2.97.0                     |
| Watch mode                   | not built (plan 010 is a draft): the README must not claim it                                 |
| Docker                       | not built yet; plan 032 (decision 1) describes Docker and never cleans it                     |
| The Trash                    | the app never empties it and never offers to (plan 037, decision 1)                           |

## Design

### 1. The order of the README

Top to bottom, about a third shorter than today:

| Block                     | Anchor                    | What                                                                                                   |
| ------------------------- | ------------------------- | ------------------------------------------------------------------------------------------------------ |
| Head                      | —                         | the logo; "Your Mac is full. See why."; the line under it; four badges; the anchors' line             |
| Picture                   | —                         | `assets/demo/overview.gif` when plan 038 has made it; absent until then (agent's call 3)               |
| Install                   | `#install`                | before a release: one line saying so and the three commands to build it; the macOS version          |
| What it finds             | `#what-it-finds`          | the generated block (section 2): the four levels and the table                                         |
| Why you can trust it      | `#why-you-can-trust-it`   | four lines, each linking to the doc that says how it is enforced                                       |
| What it does not do       | `#what-it-does-not-do`    | one honest paragraph                                                                                   |
| No warranty               | `#no-warranty`            | the agreement's words (plan 039), five short lines                                                     |
| Help it get better        | `#help-it-get-better`     | the issue templates by name; one line asking for a star                                                |
| Build and contribute      | `#build-and-contribute`   | a link to `CONTRIBUTING.md` and `docs/`; the stack in one line                                         |
| Licence                   | `#licence`                | one line, the two licence files and `NOTICE`                                                           |

**The head.**

- Logo: `assets/logo.png`, 160 px, centred (as today).
- Title `Fresher`; the pain: **Your Mac is full. See why.**; under it: "A free, open-source disk cleaner for macOS
  that explains every gigabyte in plain English, shows the risk of removing it, and deletes nothing you have not
  confirmed. Made with developers' Macs in mind."
- Badges (shields.io, static where no release exists): licence `MIT OR Apache-2.0`, `macOS 13+`, `network: none`,
  `status: pre-release` (replaced by the release badge when plan 013 publishes; deferred, section 7). Gone: stars,
  Rust, Tauri (the technology goes last, agent's call 2).
- The anchors' line: Install · What it finds · Why you can trust it · Build.

**Install, before a release.** "There is no release yet. Until there is, build it from source (macOS 13 or later,
[Rust](https://rustup.rs), Node.js 24):" and the block `git clone …`, `cd fresher`, `npm install`, `npm run build`,
with the line "the app bundle is written to `target/release/bundle/macos/`". `npm run dev` moves to Build and
contribute.

**Why you can trust it** — four lines, each one sentence and a link:

1. **It asks first.** Nothing is removed until you confirm the exact items, their size and their danger level →
   `docs/restrictions/deletion-safety.md#explicit-confirmation-only`.
2. **Trash first, and you can put it back.** Items go to the Trash; Cleanups puts them back; deleting for good is a
   separate tick → `docs/restrictions/deletion-safety.md#trash-first`.
3. **Nothing leaves your Mac.** No network requests at all: no telemetry, no crash reports, no update checks →
   `docs/restrictions/privacy.md#no-network`.
4. **Nothing in the background.** No helper, no login item, no scheduled scans; it works only while it is open →
   `docs/goal.md#8-transparent-it-does-only-what-the-user-asks`.

**What it does not do** — one paragraph: no antivirus, no "speed up your Mac", no memory cleaning; no background
helper; it never removes what it cannot name (no "Other"); it never removes anything of Docker's; it never empties
the Trash for you (you get the space back when you empty it). The Docker sentence becomes "it shows what Docker takes
and the command to free each part, and never runs it" in step 04/01 when plan 032 is on `main`.

**No warranty** — the five lines of plan 039's agreement, as written there: free and open source, no warranty; it
removes only what you confirm, so each removal is your decision; a danger level is the app's best judgement, not a
guarantee; keep a backup of anything you cannot lose; items go to the Trash and can be put back until you empty it.
Written now from plan 039's draft; step 04/01 makes them the same as the text plan 039 ships, if it is on `main`.

**Help it get better** — "The most useful report is a wrong finding": links to the three issue templates (section 4)
by their `new?template=` URL; "If Fresher helped you, a star helps others find it." — once, at the end (agent's
call 5).

**Build and contribute** — `npm install`, `npm run dev`, `npm test` in one block; "How to contribute:
[`CONTRIBUTING.md`](.github/CONTRIBUTING.md); the goal, the rules and the architecture: [`docs/`](docs/README.md)";
the stack in one sentence (a Rust core, a Tauri 2 shell, a React and TypeScript interface).

**What leaves the README**: "Why" (folded into the head), the feature bullets (folded into What it finds and Why you
can trust it), "Safety first" and "Privacy" (folded into Why you can trust it), "How it is built" (its table moves to
`docs/README.md`, section "The stack"), the contributor's long notes (into `CONTRIBUTING.md`), the levels' table
(generated into What it finds), the author line (kept: one line under Licence).

**The voice**: plain, calm, second person, no superlatives, no exclamation marks, no emoji (plan 039's guide; until it
exists, these words).

### 2. The generated block

Between two markers in `README.md`:

```markdown
<!-- what-it-finds:start — generated by `npm run readme:generate` from docs/categories; do not edit by hand -->
…
<!-- what-it-finds:end -->
```

Content, built only from `docs/categories/`:

1. A sentence with the count: "Fresher knows **N kinds** of files, each with a danger level:". Since the follow-up
   of 2026-11-25 ([`readme.md`](./readme.md), "Follow-up"): a `<details>` whose summary carries the count instead,
   items 2 and 3 folded inside it.
2. The four levels, one line each: `**Safe** — <Meaning>` from the "Meaning" column of the "Danger levels" table in
   `docs/categories/README.md`, in the order Safe, Low, Medium, High.
3. A table `| What | Where, for example | Danger |`: one row per category of the catalogue table; "What" is the
   category file's H1 linked to `docs/categories/<file>`; "Where, for example" is the file's `**Example:**` line;
   "Danger" is the level. Rows sorted by level (Safe, Low, Medium, High), then in the catalogue table's order (which
   keeps one module's rows together).

**The `**Example:**` line** (agent's call 6): a new line in every category file, right after the header line —
`**Example:** <text>` — one line, at most 80 characters of text, no `|`. It is what a developer recognizes:
`` `~/.npm/_cacache` ``, `` `node_modules` of a project untouched for 90 days ``, "Chrome and Firefox caches". The
catalogue's format (`docs/categories/README.md`, "Format") gains it, so every later plan's category file carries it,
and the check refuses a file without it. The contract test is not changed (it reads only the danger line).

**The script** `scripts/readme.ts` (with its pure parts in `scripts/readme-catalogue.ts` and
`scripts/readme-links.ts`):

- `npm run readme:generate` — rewrites the block and formats the whole README with Prettier's API and the repository's
  config, so the hook's own formatting never changes it again.
- `npm run check:readme` — computes the same text and fails, naming each problem, when the README differs, when the
  catalogue is inconsistent, or when a link or an image is wrong (section 3). It runs in `pre-push` and in CI's
  `lint` job.
- Pure functions are exported for the tests; the file system and Prettier are touched only in the entry point and in
  one `formatMarkdown` function.

**What the catalogue check refuses** (each a message naming the file):

| Refusal               | When                                                                                     |
| --------------------- | ---------------------------------------------------------------------------------------- |
| `missing-file`        | a catalogue row names a file that does not exist                                         |
| `unlisted-file`       | a `docs/categories/*.md` other than `README.md` has no catalogue row                     |
| `duplicate-row`       | two rows name the same category or the same file                                         |
| `no-title`            | the file has no `# ` line                                                                |
| `danger-mismatch`     | the file's danger level is missing or differs from its row's                             |
| `no-example`          | the file has no `**Example:**` line, or an empty one                                     |
| `bad-example`         | the example is over 80 characters or holds a `\|`                                        |
| `no-levels`           | the "Danger levels" table is missing a level                                             |
| `markers`             | the README has not exactly one start and one end marker, in that order                   |
| `stale-block`         | the README differs from what `readme:generate` would write                               |

### 3. The link and image check

Over `README.md` and every `.github/*.md`:

- **Relative links** (`[text](path)`, `<a href="path">`): the path, without `#anchor` or `?query`, resolved against
  the file's folder, exists in the working tree. A `#anchor` into a Markdown file names one of its headings by
  GitHub's slug (lower case, every character but letters, digits, spaces and hyphens dropped, spaces made hyphens).
- **Images** (`![alt](src)`, `<img src="…">`): a relative image exists and lies under `assets/`; an absolute image is
  allowed only from `https://img.shields.io/` (agent's call 7).
- **Absolute links** (`https://…`, `mailto:`) are not fetched: no network in a check (rule 6, deterministic tests).
- **Claims not yet true**: a short list in the script, each with its reason; today one entry, `watch mode`
  (plan 010 is not built). A README that names it fails (agent's call 8). The entry is removed by the plan that builds
  watch mode.

Refusals: `dead-link`, `dead-anchor`, `image-outside-assets`, `missing-image`, `remote-image`, `unbuilt-claim`.

### 4. Beside the README: GitHub's files

GitHub's own configuration, short, each pointing into `docs/` (agent's call 10). `docs/restrictions/repository.md`
gains the rule "Documentation lives in `docs/`" with these as the named exceptions.

| File                                         | What                                                                                       |
| -------------------------------------------- | ------------------------------------------------------------------------------------------ |
| `.github/CONTRIBUTING.md`                    | setup (three commands), the rules to read, Conventional Commits, the hooks, negatives-first tests, the clean room in one sentence, "a contribution is accepted under MIT OR Apache-2.0" |
| `.github/SECURITY.md`                        | what counts (a removal outside a confirmed plan, a path outside the scanned root, a network request), how to report privately (GitHub's "Report a vulnerability"), the supported version (the latest `main` until a release) |
| `.github/ISSUE_TEMPLATE/wrong-finding.yml`   | "The app offered something it should not have": the category, the path shown (with the home folder as `~`), what it really is, the danger level shown, whether it was removed |
| `.github/ISSUE_TEMPLATE/missing-folder.yml`  | "A folder Fresher should know": the path pattern, the tool that makes it, whether it comes back, a size  |
| `.github/ISSUE_TEMPLATE/bug-report.yml`      | "Something else went wrong": the macOS version, the app's version or commit, the steps, what happened |
| `.github/ISSUE_TEMPLATE/config.yml`          | `blank_issues_enabled: false`; one contact link to the security policy                     |

Plan 039's step 03 plans "the issue template" for a wrong finding. Whichever plan comes first writes it; the second
keeps it and only adds what it lacks (04/01 checks).

**The repository's description and topics** (applied by the user, agent's call 9):

- Description: "A free, open-source macOS disk cleaner that explains every gigabyte and deletes nothing you have not
  confirmed."
- Topics: `macos`, `disk-cleaner`, `disk-space`, `rust`, `tauri`, `privacy`, `developer-tools`, `open-source`.
- The command, for the user: `gh repo edit AlexSergey/fresher --description "<the description>" --add-topic macos …`.

**The social image**: `assets/social-preview.svg` (1280 × 640: the logo, "Fresher", "Your Mac is full. See why.",
the Graphite background) and `assets/social-preview.png` rendered from it with `rsvg-convert` (a tool on the
developer's Mac, listed in `docs/dependencies.md`, never shipped). GitHub takes it only through its settings page:
the user uploads it.

### 5. Testing

The code is three files: `scripts/readme-catalogue.ts` (the catalogue and the block, pure),
`scripts/readme-links.ts` (the link and image check, pure) and `scripts/readme.ts` (the entry, the file system and
Prettier). Their tests, `scripts/readme-catalogue.test.ts`, `scripts/readme-links.test.ts` and
`scripts/readme.test.ts` (`node:test`), negatives first, every test on strings and temporary folders
(`fs.mkdtempSync` under `os.tmpdir()`), never the real `docs/`:

- Negatives: each refusal of sections 2 and 3 on a made-up catalogue or README.
- Positives: the block built from a two-category catalogue (exact text); the sorting by level then catalogue order;
  the README rewritten between the markers only; a README in step passes; a relative link, an anchor, an image under
  `assets/` and a shields.io badge pass; the real repository passes (`check:readme` run in the step's done-when, not
  as a test).
- `formatMarkdown` is tested once with Prettier (a table is aligned the way the hook would align it).

No sandbox, no e2e, no coverage bar change: the scripts are outside the Rust and UI coverage (they are tested by
`test:scripts`). The finish is a look at the rendered page on GitHub, in the light and the dark theme and at a phone's
width (the user's walk).

### 6. Order and what can be done now

01 (the generator), 02 (the text), 03 (GitHub's files) can be built now. 04 waits for the other plans on `main`: it
merges `main` in, adds the `**Example:**` line to the category files that arrived (plans 031, 032, 035, 036, 037),
regenerates, adds the picture when `assets/demo/overview.gif` exists, aligns the No warranty words with plan 039 and
the Docker sentence with plan 032.

### 7. Deferred

**The README at the first release** (new file `docs/deferred/readme-at-release.md`, due when plan 013 publishes): the
Install block becomes the download link to the latest release, the package manager's line and the first-launch note;
the `status: pre-release` badge becomes the release badge; the social image re-rendered if the head changed.

## Open questions

None.

## Decisions

- 2026-11-22 — the user delegated: build autonomously, the recommended decisions and agent's calls taken (translated:
  "start implementing; work autonomously with subagents, take the recommended decisions; write into docs/deferred what
  I will need to look at — I will check everything tomorrow").
- No version bump: 0.1.0 ([`../README.md`](../README.md), "Versions").

## Agent's calls (to check)

Each with what it costs if wrong.

1. **Only true claims**: watch mode leaves the README until plan 010 is built; Docker is "never removed" now and
   "described, not cleaned" once plan 032 lands. Cost: the README promises less than the roadmap.
2. **The pain first, the technology last**: Rust and Tauri leave the head and the badges. Cost: fewer developers drawn
   by the stack alone.
3. **No picture until plan 038's GIF exists**; no interim screenshot (the e2e baselines show mock data named
   "Example caches"). Cost: the first screen has no picture until 038 lands.
4. **A "What it does not do" block**: honesty is the positioning.
5. **One short ask for a star**, at the end, not at the top; the stars badge goes.
6. **An explicit `**Example:**` line in every category file** rather than a guess from "Where it lives" (a guess picks
   `~/Library` for three files and nothing for four). Cost: one more line for every future category, enforced.
7. **Images only from `assets/` or shields.io.** Cost: a badge from another service needs the script changed.
8. **A list of claims not yet true** in the check, one entry today. Cost: a word list to keep; small.
9. **The repository's settings are the user's to apply**: the plan writes the description, the topics and the image;
   the user runs `gh repo edit` and uploads the image. Cost: one manual step.
10. **GitHub's community files live in `.github/`**, outside `docs/`, short and pointing into `docs/`; recorded as an
    exception in `restrictions/repository.md`. Cost: three short texts outside `docs/`.
11. **The table is sorted by danger, then by the catalogue's order**, all rows visible (about 37 now, more with plans
    031–037). Cost: a long table; folding it into `<details>` is one line if the user prefers.
12. **English only** for now.

## Conclusions

As built; the numbers and the changes from this design are in [`readme.md`](./readme.md), "Conclusions". Not as
designed: the README is longer by `wc -l` (the table is generated and visible), there is no picture yet, and the
`core-*` categories are table rows.
