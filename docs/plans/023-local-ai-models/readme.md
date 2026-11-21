# 023 — Local AI models

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 ·
**Modules:** modules, testing, sandbox, e2e, app, app/ui, docs

Written on 2026-11-21 from the idea [Local AI models](../../ideas/local-ai-models.md) and from a read of the code on
plan 028's head (40c669ac). Approved by the user's delegation of 2026-11-20: after plans 027 and 028, build plan 023
autonomously, take the recommended decision on every question and leave each as an agent's call; the user checks
everything together later. The design, with its agent's calls: [`00-design.md`](./00-design.md).

## Goal

Local AI models take tens of gigabytes in hidden folders no app points at. The module `local-ai-models`
(`fresher-module-local-ai-models`, group **Development**) offers, one finding per model, the models in the default
stores of **Hugging Face** (`~/.cache/huggingface/hub`: models Low, datasets Low) and **LM Studio**
(`~/.lmstudio/models` and the older `~/.cache/lm-studio/models`: Medium, since a model the user imported looks like a
download). Each finding names the model as `owner/name`, says when its files were last opened or changed and what
downloading it again takes; its size is what it frees. The model's folder goes to the Trash; no tool command runs.
Nothing outside a tool's store is claimed, and a folder without the tool's layout is not a model.

**Ollama is not in this plan** (agent's call 1): one finding per Ollama model needs the walk to read its manifests and
a removal of several files as one item, neither of which the core has. Its space stays in the storage map; the work is
[deferred](../../deferred/ollama-models.md).

Nothing in the core changes: the module is a data table (`stores.toml`), a pure decision and texts, in the shape of
`dev-caches`. Tests at every level, as plans 018 to 021 did: unit, fixture, contract, sandbox, core e2e, UI e2e, runs
by hand.

Branch: `feat/023-local-ai-models`, stacked on plan 028's head, squash-merged into `main` once when done
([`../../workflow.md`](../../workflow.md)). Depends on plans [008](../008-safe-removal/readme.md),
[009](../009-system-knowledge/readme.md), [011](../011-diagnostics/readme.md), [015](../015-module-groups/readme.md)
and [018](../018-developer-caches/readme.md) (`Interest::holding`).

## Plans

| #   | Plan                                     | Status      | Depends on | What                                                                                  |
| --- | ---------------------------------------- | ----------- | ---------- | ------------------------------------------------------------------------------------- |
| 01  | [The module](./01-module/readme.md)      | 🔨 in progress | —          | the crate, the table, the categories and texts, the names and the decision, fixtures, registration |
| 02  | [End to end](./02-end-to-end/readme.md)  | 🔨 in progress | 01         | the two case files, the core e2e flows, the UI                                        |
| 03  | [Finish](./03-finish/readme.md)          | 🔨 in progress | all        | the docs, the runs by hand, the conclusions                                           |

### Order and parallel work

```
01/01 ──┬── 01/02 ── 01/03 ──┬── 02/01 ──┬── 03/01 ── 03/02
        │                    └── 02/02 ──┤
        └── 02/03 ───────────────────────┘
```

- **Wave 1**: 01/01 alone (the crate, its table, texts and category docs).
- **Wave 2, in parallel**: 01/02 (`modules/local-ai-models/src/` only) and 02/03 (`app/ui` only, once 01/01's texts
  exist).
- **Wave 3**: 01/03 (fixtures, golden, registration).
- **Wave 4, in parallel**: 02/01 (case files, `app/tauri/tests/sandboxes.rs`) and 02/02 (`crates/e2e`).
- **Wave 5**: 03.

Which files each plan touches:

| Plan | Owns                                                                                                                  | Shared files it touches                                                                                                    |
| ---- | --------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| 01   | `modules/local-ai-models/`, `fixtures/homes/local-ai-models-*.toml`, `fixtures/golden/local-ai-models.json`, `docs/categories/local-ai-models-*.md` | `Cargo.lock`, `app/tauri/Cargo.toml`, `app/tauri/src/{compose,commands}.rs` (the module lists), `docs/categories/README.md`, `docs/dependencies.md` |
| 02   | `fixtures/sandboxes/local-ai-models{,-lm-studio-open}.toml`, `fixtures/golden/sandbox-local-ai-models*.json`, `fixtures/golden/local-ai-models-flow.json`, `crates/e2e/tests/{local_ai_models.rs,local_ai_models_home/}`, `app/ui/src/test/local-ai-models-scan.ts` | `app/tauri/tests/sandboxes.rs`, `crates/e2e/Cargo.toml`, `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/`, `docs/testing.md` |
| 03   | `docs/deferred/ollama-models.md` (checked)                                                                            | `docs/` (architecture, testing, manual-testing, categories, ideas, plans README)                                           |

`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand: regenerate them after a merge
and run the checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI;
  `modules/local-ai-models` itself at or above 90 %.
- `git diff <plan 028's head> -- crates/core crates/scanner crates/cleaner crates/engine` is empty: no core change.
- The module's contract test passes with `Catalogue::Required`: three category files with the same dangers, every text
  and every declared left-out reason used by a fixture run, every removal a matched path.
- Nothing but a model folder of a store can be offered: a Space, the hub's locks, a folder off the name shape or
  without `blobs` and `snapshots`, a loose file, a hidden publisher, a link, Ollama's store and a model file in a
  project are never claimed (the fixtures' and the case files' negatives).
- `npm run sandbox:test` passes for both case files; it fails when a case's expectation (a danger, a left-out's items)
  is changed on purpose (done once by hand per kind, written in 02/01).
- `npm run test:e2e` passes with the core flows and the UI scenarios.
- `npm run dev:sandbox -- local-ai-models` and `-- local-ai-models-lm-studio-open`: every line of `EXPECTED.md` holds by
  hand.
- On the developer's Mac (`npm run dev`, a dry run): every offered model checked by hand against its store (03/02).

## Ideas and deferred items this touches

- [Local AI models](../../ideas/local-ai-models.md): becomes this plan for Hugging Face and LM Studio; Ollama is
  deferred; the open questions are answered (Trash, not the tool's command; datasets a category of their own); other
  apps' models stay ideas.
- [Ollama's models](../../deferred/ollama-models.md): a new deferred item, written with this plan.
- [Developer environment cleaner](../../ideas/developer-environment-cleaner.md),
  [large files](../../ideas/large-files-finder.md): touched, not changed (a model in a project stays a user file).

## Risks

- **A link from the user's folders into the cache** breaks until the model is downloaded again: not seen by the walk;
  said by the category's text; the Trash keeps it recoverable.
- **An imported LM Studio model** does not come back by download: Medium, and the texts say so.
- **LM Studio's bundle id and its older place** are "to verify" in 03/02: a wrong id offers its models while it runs
  (the cleaner still skips the files it holds open).

## Agent's calls (summary)

The full list with costs is in [`00-design.md`](./00-design.md), "Agent's calls (to check)":

1. Ollama is deferred: per-model findings need manifest reads and a several-file removal the core lacks.
2. The model's folder goes to the Trash; no tool command.
3. Only the default places; no environment variable or tool config is read.
4. One finding per repository folder; freed equals listed (blobs belong to one repository).
5. Hugging Face datasets are a category of their own (Low); Spaces are never claimed.
6. LM Studio's models are Medium (an import looks like a download); Hugging Face's Low.
7. LM Studio's models are kept while LM Studio is open; open files are the cleaner's check.
8. No age threshold and no setting; the date shown is the newest change or file access.
9. `hub/*` is claimed whole and routed by name; the hub's locks and Spaces are summed, never claimed.
10. Signs: a Hugging Face repository holds `blobs` and `snapshots`; LM Studio's place is its own sign.
11. One module `local-ai-models`, group Development, three categories.
12. Links into the cache from the user's folders are not looked for; the text names the effect.
13. A folder read in part, or with no time, is not claimed.
14. No new dependency.
15. No new UI component.

## Open questions

None.

## Conclusions

Built on `feat/023-local-ai-models` in 11 commits over plan 028's head (bb682a65), **awaiting the user's walk**: the plan
is not marked done until the user has looked at the result (the list is
[`03-finish/02-run-and-conclusions.md`](./03-finish/02-run-and-conclusions.md)). The branch is stacked on
`feat/028-ignore-item` and is squash-merged after it.

**What was built.** The module `local-ai-models` (group Development) offers one finding per model in three categories:
Hugging Face models (Low), Hugging Face datasets (Low) and LM Studio models (Medium), read from default places only
(`~/.cache/huggingface/hub`, `~/.lmstudio/models`, `~/.cache/lm-studio/models`). The model's name comes from its
folder's path by components; a folder off the tool's shape or without its sign is not claimed; a Space is never
claimed and the hub's locks are only summed. The decision is a pure function of nine rows; each finding says when the
model was last opened or changed and what downloading it again takes, and its removal is the folder moved to the
Trash. Around it: the fixtures and a golden report, two sandbox cases (`local-ai-models`,
`local-ai-models-lm-studio-open`), the core e2e flows (default roots reach the hidden folders, Trash and put back byte
for byte, a held-open model skipped, a link kept as a link, a restore at launch) and the UI cases on two mock scripts.
`git diff bb682a65 HEAD -- crates/core crates/scanner crates/cleaner crates/engine` is empty: no core, scanner,
cleaner or engine code changed.

**Numbers on the last code commit, on the reference Mac.**

| Check                                | Result                                                                               |
| ------------------------------------ | ------------------------------------------------------------------------------------ |
| `npm run pre-push`                   | green: lint, guards, knip, machete, deny, tests with coverage                         |
| Rust tests and coverage              | 3,372 passed (5 skipped); the floor of 90 % held (plan 028: 3,290)                    |
| UI unit tests and coverage           | 1,150 passed in 130 files; 98.59 % of lines (floor 90 %)                              |
| Playwright gallery, WebKit, Chromium | 467 passed in one run, no flake; `git status` clean after it (plan 028: 449)          |
| `npm run test:e2e:core`              | 116 passed, 2 skipped (plan 028: 110)                                                 |
| Perf                                 | not run: a module plan does not touch the hot paths (docs/testing.md, 2026-11-21)     |
| What the module offered on the Mac   | pending: the dry run is the user's                                                    |

A first Playwright run failed three specs in both browsers because a fix of the final review made the mock's left-out
sizes zero; the core sums the walked sizes and the UI shows allocated, so the line read "0 bytes". The fix was
reverted (a deviation from the review's fix 2) and the rerun is green.

**What changed from the design and why.**

1. The verdict of the decision holds the category as `&'static str`, so the decision stays pure on plain values.
2. An LM Studio finding's identity is `<row id>/<publisher>/<model>`, not the folder name alone: two publishers can ship
   one model name. The design's section 5 says so.
3. The module has no page of its own (plans 027 and 028): the UI cases run on Scan's cards and Details, with two mock
   scripts because an item is never both offered and left out (LM Studio open leaves out all its models).
4. The core e2e runs on the engine's default scan root; a first version overrode the roots and a review removed it, so
   the flows prove that `.cache` and `.lmstudio` are reached by default.
5. The step's scan-time check was dropped: no benchmarks for a module plan.

**The calls taken while building (for the user to look at).**

- **Ollama is deferred** ([`ollama-models`](../../deferred/ollama-models.md)): its models need the walk to read
  manifests and one finding removes several paths. The next plans build it; until then the most common model store is
  not offered. Its folder is only shown in the storage map as the user's.
- **LM Studio models are Medium**: a model the user imported by hand cannot be told from a download.
- **Default places only, no environment variables**: `HF_HOME`, `HF_HUB_CACHE` and the like are not read; a store moved
  by them is not found.
- **No age threshold**: every model is offered with the date it was last opened or changed in its reason; the user
  decides.
- **The folder goes to the Trash, no tool command**: Hugging Face's cache has no index beside its folders, and a tool
  never runs in a sandbox.
- **LM Studio's models are kept while LM Studio is open** (or when the running apps cannot be listed); its bundle id
  is still "to verify" on the real Mac, as is whether the check sees a background server after the window closes.
- **The identity of an LM Studio model includes the publisher** (see 2 above).

**Left open (parked minors, none a defect the user can hit):** the special-entry and no-system reasons have no UI case;
the first negative UI test packs three flows; the golden's allocated and reclaimable sizes are 0 as the other modules'
(a fixture tree has no disk blocks); one `unwrap` on a `find` in a test loop; a cosmetic padding in the LM Studio
category doc's table.

**The user's walk.** Pending. The sandbox boxes, the dry run on the real Mac and its facts "to verify" (the bundle id,
LM Studio's older place, whether a last-used date exists, a download in progress, a model imported by hand, a
background server) are unchecked in
[`03-finish/02-run-and-conclusions.md`](./03-finish/02-run-and-conclusions.md); the text, table and category docs are
corrected where the real Mac differs. This plan becomes ✔️ done when it passes. The user checks plans 027, 028, 023
and 024 together (the delegation of 2026-11-20).

