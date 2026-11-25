# 042/08/01 — The core and module texts

**Status:** 🔨 awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-24 · **Modules:** core, modules (locales), sandbox, app/ui (fixtures), docs

Every text of the core's and the modules' dictionaries, rewritten shorter and plainer; the trial and demo modules'
items get readable names. The design: [`../00-design.md`](../00-design.md), section 12; the feedback's items 5 to 8.

## Read first

`docs/writing.md` (all), `docs/rules.md` rules 3 and 7, `crates/core/locales/en.json`, every `modules/*/locales/
en.json`, `docs/categories/` (their quoted texts), `modules/trial/` and `fixtures/sandboxes/trial.toml` (the
`fresher-trial-*` names a person sees), `modules/demo/`, `rg -l "core\.|<module>\." app/ui/src/test` (fixtures that
copy texts), the sandbox goldens that quote a name.

## The procedure

1. For each category (core and modules): `name` 2 to 4 words, a noun phrase a person knows ("App caches", not
   "Caches of your apps"); `line` at most 50 characters, no full stop (01/04's draft, made plainer); `what`, `made-by`,
   `if-removed`, `comes-back` one sentence each, two only when the second says what you keep ("Nothing is lost. Some
   apps open a little slower once."). The mockups' texts for App caches, Xcode build data, Old installers, Unused Node
   versions and Builds of old projects are the model.
2. The storage texts: the five summary parts (01/03) checked; the detailed parts' names stay plain where a screen still
   shows them.
3. Every refusal, left-out, skip, failure and result text: shorter, in the shapes of `docs/writing.md` section 2.
4. The trial module: its items read as things a person could own ("Sample app cache", "Old sample download", "Sample
   tool images" — names the implementer picks in this voice): change the names the case file writes and any title the
   module builds; the trial sandbox's golden report and `EXPECTED.md` follow. The demo module's texts likewise.
5. `docs/categories/*.md`: each heading and quoted text follows its dictionary (`# <name>` is the name).
6. `docs/writing.md`: sections 1 to 3 and 7 gain what the pass applied: the group names (Safe to remove, Usually safe,
   Look before removing; the level names stay in the code and the confirmation), the five part names, the `line`
   shape, and a word list of replaced terms: "Swap and sleep image" → "System space"; "Explained" → "Can be freed";
   "Your folders" → "Your files and apps"; "danger zone" → "Look before removing"; "Low risk", "Medium risk" → the
   group names; "Comes back by itself when needed" → "Apps make these again on their own". (The checked lengths come in
   08/03.)
7. The UI's fixtures that copy a changed text follow it.

## Tests

No new logic: every module's contract test, the core's dictionary tests, the sandbox goldens and `npm run
check:texts` (the current rules) hold the texts.

## Checks

`CARGO_TARGET_DIR=<target> cargo nextest run --workspace`; `npm run sandbox:test`; `npm run check:texts`;
`npx vitest run`; `npx playwright test e2e/visual.spec.ts --project webkit --project chromium --update-snapshots`
where a core text shows (look at each); `git diff main -- modules` touches only locales, the trial and demo modules'
names and their tests and fixtures.

Commit: `docs(docs): plainer texts in the core and the modules` — or `feat(core): ...` if Rust code changed (the
trial's names).

## Conclusions

- 271 core and module texts rewritten (keys, placeholders and levels unchanged); the review round fixed 11 more
  ("Your files and apps" for the finer part, plainer and truer answers). The trial's items are "Sample app cache" and
  "Old sample downloads"; the demo's names stay; the trial tool's command stays `fresher-trial-tool` (ruling: a
  common name could run a stranger's program). Goldens and the README catalogue regenerated.
- Before and after, the 15 most important:

  | Key | Before | After |
  | --- | --- | --- |
  | `core.storage.part.swap-and-sleep.name` | Swap and sleep image | Memory kept on disk |
  | `core.storage.part.explained.name` | Explained | Can be freed |
  | `core.storage.part.your-folders.name` | Your folders | Your files and apps |
  | `app-caches.library-caches.name` | Caches of your apps | App caches |
  | `app-caches.library-caches.if-removed` | What is inside the folder goes to the Trash … | Nothing is lost. Some apps open a little slower once. |
  | `app-caches.library-caches.comes-back` | Yes, the app makes it again as it needs it. … | Yes, by itself, the next time the app needs it. |
  | `downloads.installers.name` | Installers of installed apps | Old installers |
  | `unused-toolchains.node.name` | Unused Node versions (nvm) | Unused Node versions |
  | `stale-projects.build-output.name` | Build output of old projects | Builds of old projects |
  | `jetbrains.settings.name` | Settings and plugins of older JetBrains IDEs | Old JetBrains IDE settings |
  | `dev-caches.npm.if-removed` | It goes to the Trash; the space comes back when you empty it. … | Nothing is lost. Your projects and the packages in them keep working. |
  | `dev-caches.cargo.name` | Cargo's downloaded crates | Cargo's download cache |
  | `xcode.device-support.name` | Device support files | Device debug files |
  | `*.left-out.special` | It holds a connection a running program may be using, so it is kept. | A running program may be using it, so it is kept. |
  | the trial's items | fresher-trial-cache / fresher-trial-kept | Sample app cache / Old sample downloads |

- Detail shortened away, listed for the user: App caches' note on music and maps kept for offline use.
