# 039/04/02 — The core's and the modules' texts

**Status:** 🔨 awaiting the user's walk · **Version:** 0.2.0 · **Date:** 2026-11-23 · **Modules:** core, modules, docs

A pass over the core's dictionary and every module's, as they stand when this step runs, against
`docs/writing.md` (written by 04/01). A procedure, not a list: it reads whatever modules exist then. The design:
[`../00-design.md`](../00-design.md), section 4.

## What may and may not change

- **May change**: the text of a key, to read in the guide's voice and shape, within its limits.
- **Never changes**: a key; a placeholder (`{name}`, `{size}`, …) or the set of them in a text; a plural pair's shape
  (`.one` and `.other` both stay); a danger level; a category id; what a text promises about safety (a text that says
  something is kept, skipped or comes back keeps saying exactly that). The agreement's lines
  (`core.agreement.*`) change only by raising `AGREEMENT_VERSION` with a `changes` text — this pass leaves them as they
  are unless the check reports them, and then says so in the conclusions.
- A text whose meaning looks wrong (it promises what the code does not do) is **not** rewritten in passing: it is
  listed in the conclusions and in the user's review list, and fixed in its own commit only if the code proves the
  text wrong.

## The procedure

1. List the dictionaries: `ls crates/core/locales/en.json modules/*/locales/en.json`. Run `npm run check:texts` and
   keep its output (the core's and the modules' part of 04/01's work list, refreshed).
2. **The core's dictionary**, group by group in the order of `crates/core/src/texts.rs` (`core_keys`): danger names and
   meanings and the danger parts; groups; completeness; errors (each: what was refused, "Nothing was removed." where a
   cleanup was refused, what to do); scan outcomes; capabilities and access (what the permission unlocks, how to give
   it); removability; cleanup outcomes, skips and their "how" texts, failures, put back, blocked reasons; storage
   texts; the core's settings (label and description, rule 7). Fix every check violation and read every text against
   its shape. On 740cf3c3 the core's groups also hold the core's own categories (`core.own-file`, plan 035;
   `core.uninstall-keep-data` and `core.uninstall-everything`, plan 036: five parts each, docs
   `docs/categories/core-*.md`), the Largest kinds and places, the apps' offers and not-offered reasons, the uninstall
   titles, the Trash and the insight texts.
3. **Each module**, in alphabetical order of `modules/` (on 740cf3c3: app-caches, app-leftovers, demo, dev-caches,
   docker, downloads, jetbrains, local-ai-models, logs, stale-projects, temp, trial, unused-toolchains, xcode; `demo`
   proves the module contract and `trial` is for development builds; both get the same pass): its categories' five parts (compare with
   `docs/categories/<file>.md`: a doc that quotes a text is updated in the same commit, and the doc's table stays true);
   its settings and choices (rule 7: what it does, what you keep or lose); its titles and reasons (the item named, the
   size or age from a placeholder); its left-out and problem texts (what stopped Fresher, what you can do). Docker
   has no category (plan 032: described, never cleaned): its parts' `label`, `detail`, `none` and `effect`, and its
   insight texts, say what Docker keeps and the command that frees it, never that Fresher removes it.
4. Run each module's contract and texts-used tests (`cargo nextest run -p fresher-module-<id>`) and the core's
   dictionary tests (`cargo nextest run -p fresher-core texts`): a key or a placeholder changed by mistake fails them.
   Then `cargo nextest run --workspace`: a test that asserts a literal text is updated to the new text (never removed).
5. Golden files: if any `fixtures/golden/*.json` holds a rendered text (not a key), regenerate it with the command its
   test names and review the diff line by line. The demo scenarios: a changed text that a step of
   `fixtures/demos/scenarios/*.toml` names (on 740cf3c3: `core.danger.safe.zone` "Safe to remove",
   `dev-caches.xcode-derived-data.name` "Xcode build data" inside "Select …") is changed in the scenario too, and
   `CARGO_TARGET_DIR=<target> cargo nextest run -p fresher-sandbox scenario` passes (its trace tests name keys).
6. Look at the changed texts where they show: `npm run test:e2e:ui -- --update-snapshots` for the screenshot specs
   whose images hold these texts (the category Details screens, the confirmation, Settings), and look at every
   changed image before committing. The gallery (`npm run dev` of the gallery page, as `docs/testing.md` says) for the
   components that render core texts.
7. `npm run check:texts` reports nothing for the core's and the modules' dictionaries (the interface's may still
   report: 04/03).

## Files

- `crates/core/locales/en.json`, every `modules/*/locales/en.json` — the texts.
- `docs/categories/*.md` — where a quoted text changed.
- `fixtures/demos/scenarios/*.toml` and `crates/sandbox/src/scenario/traced.rs`'s tests — where a target's text
  changed.
- Tests asserting a literal text, golden files and screenshot baselines — as the texts changed.

## Checks

- `CARGO_TARGET_DIR=<target> cargo nextest run --workspace`; `npm run sandbox:test`; `npm run test:ui`;
  `npm run test:e2e:ui` (the baselines looked at); `npm run check:texts` clean for these files;
  `git diff -- '*.rs' | grep '^[-+]' | grep -v 'locales'` shows only literal-text test updates (no logic changed).

## Conclusions

Nine texts changed in all. The check reported 7 violations in the core and the modules and now reports none there
(15 remain, all in `app/ui`, for 04/03). Per dictionary: core 2, app-leftovers 2, jetbrains 5; the other modules 0.
Every text of the core (351 lines) and of the 14 modules was read against `docs/writing.md`; apart from the check's
findings and the JetBrains names below, none broke the voice, the shapes or the glossary.

- Core: `failed.delete-failed` and `failed.partly-deleted` now say "permanently" (they describe the permanent delete).
- app-leftovers: the two choice descriptions say "Removes" instead of "Deletes"; what they promise is unchanged.
- jetbrains: the four category names now read "Caches / Logs / Settings and plugins of older JetBrains IDEs" and
  "Data of JetBrains IDEs no longer installed" (all under 48); `left-out.used-recently` is 240 or less and opens with
  "Fresher could not find an installed IDE for this data".
- Docs updated: the titles of the four `docs/categories/jetbrains-*.md`. The e2e specs that name those categories
  follow, and eight screenshot baselines (chromium and webkit: category details of app-leftovers, app-leftovers with
  Keep settings, jetbrains, and the Medium confirmation) were regenerated and looked at: only the changed words moved,
  image sizes are unchanged. No golden file, sandbox expectation or demo scenario held a changed text.

Texts left as they are, for the user's review list (not rewritten, per the constraints):

- Quotes of the old "Deletes ..." wording remain in `docs/rules.md` (rule 7's example "Deletes only caches and logs.
  ...") and in `docs/ideas/uninstalled-app-leftovers.md`; they are examples and an idea, not the dictionary.
- Numbers typed into a sentence, against the guide's "numbers come from the core": `core.error.plan-gone` (10
  minutes), `downloads.macos-installers.what` (12 to 15 GB), `downloads.large-untouched.what` (1 GB),
  `unused-toolchains.rust.comes-back` ("a few hundred megabytes"), the "at least 7" and "at least 30" days in the
  `*-days` descriptions of app-leftovers, downloads, jetbrains and stale-projects, `logs`/`temp` "7 days" and "three
  days", `stale-projects.left-out.cache-recent` (7 days). A placeholder would change the set of placeholders.
- Meaning to check against the code or the tool: `jetbrains.caches.comes-back` starts "No." yet says the IDE rebuilds
  them; `temp.old-files.comes-back` states what macOS removes each day; `docker.part.volumes.effect` says which volumes
  the printed command removes.
