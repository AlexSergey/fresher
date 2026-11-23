# 039/04/01 — The guide and the check

**Status:** 🔨 awaiting the user's walk · **Version:** 0.2.0 · **Date:** 2026-11-23 · **Modules:** scripts, docs

`docs/writing.md`, the one voice of every text, and `scripts/check-texts.ts`, the check that holds the rules a script
can hold. The check is written and tested here and run by hand; it joins pre-push in 04/03, once the pass has made it
report nothing. The design: [`../00-design.md`](../00-design.md), section 4.

## Read first

`docs/rules.md` (rule 3: a category's five parts; rule 7: every choice's description), `crates/testing/src/contract.rs`
(what the module contract already holds over texts: do not duplicate its errors, complement them),
`scripts/check-crate-deps.ts` and its test (the shape of a guard: pure functions exported for the test, a thin `main`
at the bottom), `package.json` (`scripts`), `docs/quality.md` (the guards' table and the pre-push list),
`docs/README.md` (the index), and a sample of each dictionary: `crates/core/locales/en.json`,
`app/ui/src/locales/en.json`, two `modules/*/locales/en.json` and `modules/docker/locales/en.json` (plan 032: its parts
have a `label`, a `detail`, a `none` and an `effect`, no `description`). On 740cf3c3 there are 16 dictionaries and
1164 texts (the design, "What already exists").

## Files

- `docs/writing.md` (new). Sections, each short, each rule the check enforces marked "(checked)":
  1. **The voice** — plain, calm, second person ("you", "your"), present tense, active; "Fresher" for the app; no
     jargon a person who never opened Terminal would not know (a technical name only where it is the thing's name,
     such as a tool's); no exclamation marks (checked); no fear words (checked: the banned list); numbers and sizes
     come from the core, never typed into a sentence.
  2. **The shapes** — a category's five parts (`name`: what it is, a noun phrase; `what`: what the files are;
     `made-by`: which app or part of macOS makes them; `if-removed`: what happens, what you keep, what you lose;
     `comes-back`: whether and when it returns) (all five present: checked); a choice and a setting (`label`: what it
     is or does; `description`: what it does and what you keep or lose, one or two sentences) (a description for
     every label: checked); a left-out or skip reason (what stopped Fresher, then what you can do); a result (what
     happened, what did not, what next); a refusal (what was refused, that nothing was removed when it is a cleanup,
     what to do); a hint (what the control does or what the number counts, in one sentence); the agreement (its
     lines change only with a new version and a "what changed" text: see
     `restrictions/deletion-safety.md`, "The agreement").
  3. **The glossary** — one word for one thing: remove (the general act), move to the Trash, delete permanently
     ("delete" only for a permanent delete: checked), empty (a folder's contents go, the folder stays), put back,
     left out (kept out of the list before a cleanup), skipped (kept at the cleanup), danger level, Safe, Low, Medium,
     High, danger zone, the Trash ("Trash" capitalised: checked), folder (never "directory": checked).
  4. **Lengths** (checked), by the key's last segment (a plural `.one`/`.other` counts as the segment before it):
     `name` 48, `label` 48, `title` 64, `made-by` 160, `description` 200, `what` 220, `comes-back` 220, `if-removed`
     240, any other 240 characters, placeholders counted as written.
  5. **Form** (checked) — no double space, no space at either end; `description`, `what`, `made-by`, `if-removed`,
     `comes-back`, and the texts under `left-out.`, `skip.` and `failed.` end with ".", "…" or ":" (a text followed by
     a list or an address).
  6. **The banned words** (checked) — whole words, any case: "junk", "garbage", "simply", "easily", "please", "oops",
     "sorry", "click here", "at risk", "you must", "directory". The list is the script's `BANNED`; this section mirrors
     it.
  7. **Examples** — three before/after pairs written for generic cases (a cache category, a setting, a skip reason);
     no real user's paths or names.
  8. **How it is checked** — `npm run check:texts`; the module contract (`crates/testing/src/contract.rs`) for what it
     holds; exemptions sit in the script with their reason, and a new one is a reviewed change.
- `scripts/check-texts.ts` (new) — exports pure functions for the test: `type Dictionary = Readonly<Record<string,
  string>>`, `type Violation = { readonly file: string; readonly key: string; readonly rule: RuleId; readonly message:
  string }`, `type RuleId = 'banned' | 'category-parts' | 'delete' | 'description' | 'exclamation' | 'form' | 'length'
  | 'sentence' | 'trash'`, `checkDictionary(file: string, dictionary: Dictionary): Violation[]`; constants `LIMITS`,
  `BANNED`, `SENTENCE_KINDS`, `EXEMPT: ReadonlyMap<string, string>` (key → reason; empty unless a reviewed reason
  exists). The rules:
  - `exclamation`: no `!` in a text.
  - `length`: the limits above.
  - `banned`: the list above, whole words, case-insensitive.
  - `delete`: a text matching `/\bdelet/i` must contain "permanent" (any case) or have a key segment among `delete`,
    `deleted`, `deleting`, `permanent`.
  - `trash`: the word "trash" as a whole word in lower case is a violation (the Trash is a name); "Trash" passes.
  - `description`: every key ending `.label` outside `ui.` has its `.description`, unless its prefix also has an
    `.effect` (a part described by its `detail` and `effect`, as Docker's four `docker.part.*`; design call 17).
  - `category-parts`: a prefix with any of `.what`, `.made-by`, `.if-removed`, `.comes-back` has all four and its
    `.name`.
  - `form`: no double space; no leading or trailing space.
  - `sentence`: the `SENTENCE_KINDS` end with ".", "…" or ":".
  The `main` part (only when run as a script) reads `crates/core/locales/en.json`, `app/ui/src/locales/en.json` and
  every `modules/*/locales/en.json` (listed with `readdirSync`, so a module added later is read), prints one line per
  violation (`<file>: <key>: <rule>: <message>`) and a count, and exits 1 when there is any.
- `scripts/check-texts.test.ts` (new) — `node:test`, `describe('check-texts')` with `describe('negative cases')` first:
  one test per rule with a dictionary that breaks it (names: "reports an exclamation mark", "reports a name over 48
  characters", "counts a plural by the segment before it", "reports a banned word as a whole word only", "reports
  delete outside a permanent delete", "reports a lower-case trash", "reports a label without a description outside
  ui", "reports a category missing a part", "reports a double space", "reports a description that does not end as a
  sentence"); then `describe('positive cases')`: "passes a dictionary that keeps every rule", "lets ui labels go
  without a description", "lets a part label with its effect go without a description", "accepts delete in a
  permanent text", "skips an exempt key".
- `package.json` — `"check:texts": "node scripts/check-texts.ts"`; not yet in `pre-push` (04/03 adds it).
- `docs/quality.md` — the guards' table gains the text check (what it reports, `npm run check:texts`), "joins pre-push
  in plan 039, step 04/03" until then.
- `docs/README.md` — the index row for `writing.md`. `docs/rules.md`, rule 7 — one sentence: the voice and the shapes
  are in `writing.md` (a relative link).

## Checks

- `node --test scripts/check-texts.test.ts`; `npm run test:scripts`; `npm run lint` (ESLint and `tsc` cover
  `scripts/`); `npm run knip`; `npm run lint` of the Markdown and its links; `npx prettier --check` on the changed files.
- `npm run check:texts` run once by hand: its violation count per rule and per file written in this step's
  conclusions (the work list of 04/02 and 04/03). It is expected to fail now.

## Conclusions

As built (commit 3da3ff93): `docs/writing.md` (voice, shapes, glossary, lengths, form, banned words, examples) and
`scripts/check-texts.ts` with its test, run by `npm run check:texts`; not yet in pre-push (04/03 adds it once the
check reports nothing). The check reads the core's, the interface's and every module's English dictionary.

The first run found 22 violations. Per rule: category-parts 4, delete 4, length 3, sentence 9, trash 2.
Per file: `app/ui` 15, `crates/core` 2, `modules/app-leftovers` 2, `modules/jetbrains` 3.

| File                       | Key                                              | Rule           |
| -------------------------- | ------------------------------------------------ | -------------- |
| core                       | `core.cleanup.failed.delete-failed`              | delete         |
| core                       | `core.cleanup.failed.partly-deleted`             | delete         |
| app-leftovers              | `app-leftovers.choice.everything.description`    | delete         |
| app-leftovers              | `app-leftovers.choice.keep-settings.description` | delete         |
| jetbrains                  | `jetbrains.settings.name` (52)                   | length         |
| jetbrains                  | `jetbrains.uninstalled.name` (51)                | length         |
| jetbrains                  | `jetbrains.left-out.used-recently` (246)         | length         |
| ui                         | `ui.build-banner.sandbox`, `.sandbox-simulated`  | trash          |
| ui                         | `ui.confirm.left-out`, `ui.finding-list.failed.title`, `ui.scan-hero.failed`, `ui.scan-progress.module.failed`, `ui.screen.scan.left-out.title` | sentence |
| ui                         | `ui.screen.category.{what,made-by,if-removed,comes-back}` | category-parts and sentence |

Work list: the first seven rows are 04/02 (fixed there); the `ui` rows are 04/03.
