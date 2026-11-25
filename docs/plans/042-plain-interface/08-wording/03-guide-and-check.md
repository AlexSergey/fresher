# 042/08/03 — The guide and the check

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** scripts, docs

`npm run check:texts` holds the new rules, now that every text passes them: a category's `line` and its name's words.
The design: [`../00-design.md`](../00-design.md), section 12.

## Read first

`scripts/check-texts.ts` (`LIMITS`, `SENTENCE_KINDS`, `PART_SEGMENTS`, `EXEMPT`), `scripts/check-texts.test.ts`,
`docs/writing.md` sections 4, 5 and 8, `docs/quality.md` (the check's row).

## Files

- `scripts/check-texts.ts` — `LIMITS.line = 50`; a `line` must not end with "." (a form rule); a category's `name` (a
  key whose prefix has the category's texts) has at most 4 words (`NAME_WORDS = 4`); messages in the script's style.
- `scripts/check-texts.test.ts` — negative first: "a line over 50 characters fails", "a line ending with a full stop
  fails", "a category name of five words fails"; positive: "a line of 50 passes", "a four-word name passes", "a
  `ui.` name is not counted in words".
- `docs/writing.md` — section 4: `line` 50 and the name's 4 words (checked); section 5: a `line` has no full stop
  (checked); section 2: the category's six parts.

## Checks

`npm run test:scripts`; `npm run check:texts` reports nothing; `npx prettier --check docs/writing.md`.

Commit: `feat(repo): the text check holds the line and the name's words` (scope `repo`, as the scripts' changes are).

## Conclusions

- `check:texts`: a `line` at most 50 characters (`LIMITS.line`) and no full stop (rule `form`); a category name at
  most four words (`NAME_WORDS`, rule `name-words`, not for `ui.` texts); tests negative first. `writing.md` and
  `quality.md` say so. The last texts reworded to pass; 0 violations.
