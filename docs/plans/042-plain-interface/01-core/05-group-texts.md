# 042/01/05 — The group texts

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** core, app/ui (fixtures)

The groups people read replace the level words: the zone keys get the group names, the meaning keys the group lines.
No key is added or removed here (039's `core.danger.part.*` go in 04/02, with the Review parts that read them). The
design: [`../00-design.md`](../00-design.md), section 4; agent's call 2. Mockup:
[`scan.dc.html`](../../../mockups/0.1.0/scan.dc.html) (the group headings).

## Read first

`crates/core/locales/en.json` (`core.danger.*`), `crates/core/src/texts.rs`, `scripts/check-texts.ts` (`SENTENCE_KINDS`:
`meaning` is not a sentence kind, so a line without a full stop passes), `rg "core\.danger\." app/ui/src` (the
fixtures that copy these texts: `app/ui/src/test/*.ts`), `docs/writing.md`.

## Files

- `crates/core/locales/en.json` — exactly (no full stop: they are lines):
  - `core.danger.safe.zone` "Safe to remove"; `core.danger.safe.meaning` "Apps make these again on their own"
  - `core.danger.low.zone` "Usually safe"; `core.danger.low.meaning` "You may need to download them again"
  - `core.danger.medium.zone` "Look before removing"; `core.danger.medium.meaning` "May hold something you still want"
  - `core.danger.high.zone` "Only if you are sure"; `core.danger.high.meaning` "May hold the only copy of your data"
  - `core.danger.<level>.name` (Safe, Low, Medium, High) unchanged: the code and the confirmation still name levels.
- The UI fixtures that copy these eight texts (`app/ui/src/test/{storage-map,finding-fixtures,demo-scan,...}.ts`) take
  the new texts, so the unit tests read what the core says.
- `docs/categories/README.md` — the "Danger levels" table gains a column "Shown as" with the four group names.

## Tests

No new logic. The core's dictionary tests (every key of `core_keys` present) and `npm run check:texts` hold the texts;
the UI unit tests and the gallery's baselines that show a zone name or a meaning are updated where they change (look at
each PNG).

## Checks

`CARGO_TARGET_DIR=<target> cargo nextest run -p fresher-core`; `npm run check:texts`; `npx vitest run` (the whole UI
suite: fixtures are shared); `npx playwright test e2e/visual.spec.ts e2e/gallery.spec.ts --project chromium --project
webkit --project gallery` with `--update-snapshots` only for what changed.

Commit: `feat(core): the group names and lines people read`.

## Conclusions

- The four groups' `zone` and `meaning` texts as the design wrote them; the level names stay. The catalogue's
  "Shown as" column and the UI's fixtures follow; 101 baselines changed with the group names.
