# 029 — Ollama's models

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 ·
**Modules:** core, engine, cleaner, store, testing, modules, sandbox, e2e, app, app/ui, docs

Written on 2026-11-21 from the deferred item "Ollama's models" (its file removed by this plan) and from a read of the
code on plan 024's design commit (39ec314f), with plan 024's reader in progress. Approved by the user's choice of 2026-11-21: after
plan 024, build Ollama's models autonomously, take the recommended decision on every question and leave each as an
agent's call; the user checks everything later. The design, with its agent's calls: [`00-design.md`](./00-design.md).

## Goal

Ollama's models are the largest files in many developers' homes, and the one store plan 023 could not offer: a model
is a small manifest plus the layer files it names in a shared folder, and models share layers. This plan offers them,
one finding per model, in the module `local-ai-models` (category **Ollama models**, Medium): the model's manifest and
the layers no other model names go to the Trash **together, as one confirmed item**. Its size is what it frees, and its
reason says the rest: "It is 14 GB in all, of which 9 GB is freed; the rest is shared with `llama3:70b` and stays."
Tags that name the same files are one finding. When a manifest cannot be read, a download is in progress or an entry of
the store is not the user's, nothing of Ollama is offered and the module says why.

**The core gains one capability first** (plan 01): a removal of several paths as one item
(`Removal::MoveTogether`). Every path passes the guard before the first one moves, so a model the Ollama server has
loaded (its layer mapped) moves nothing; the subject goes first; every moved path is journalled (the journal's version
2); put back brings the other paths back first and the subject last, only when all of them came back. The confirmation
lists the other paths on demand.

Manifests are read through plan 024's `cx.system().small_file` in `finish`. **The walk, the scanner and the scan cache
do not change, so no scan bench is due**; the UI change is a line of the confirm dialog, not the scan screen, a list
or motion, so no performance run either ([`../../testing.md`](../../testing.md), "Performance").

Branch: `feat/029-ollama-models`, stacked on plan 024's head, squash-merged into `main` once when done
([`../../workflow.md`](../../workflow.md)). Depends on plans [008](../008-safe-removal/readme.md) (the cleaner and its
journal), [009](../009-system-knowledge/readme.md) (open and mapped files), [011](../011-diagnostics/readme.md)
(left-outs), [023](../023-local-ai-models/readme.md) (the module) and [024](../024-unused-toolchains/readme.md)
(`small_file`, file text in the test worlds).

## Plans

| #   | Plan                                                   | Status      | Depends on | What                                                                                      |
| --- | ------------------------------------------------------ | ----------- | ---------- | ----------------------------------------------------------------------------------------- |
| 01  | [Several paths as one item](./01-several-paths/readme.md) | 🔨 in progress | —       | the core's types and engine, the cleaner, the journal and put back, the confirmation      |
| 02  | [Ollama in the module](./02-module/readme.md)          | 🔨 in progress | 01/01      | the store row, the manifests, the layers and the decision, the texts, fixtures            |
| 03  | [End to end](./03-end-to-end/readme.md)                | 🔨 in progress | 01, 02     | the three case files, the core e2e flows, the UI                                          |
| 04  | [Finish](./04-finish/readme.md)                        | 🔨 in progress | all        | the docs, the runs by hand, the conclusions                                               |

### Order and parallel work

```
01/01 ──┬── 01/02 ─────────────────────────┐
        ├── 01/03 ─────────────────────────┤
        ├── 01/04 ─────────────────────────┼── 03/01 ──┬── 04/01 ── 04/02
        └── 02/01 ── 02/02 ── 02/03 ───────┤── 03/02 ──┤
                                           └── 03/03 ──┘
```

- **Wave 1**: 01/01 (the core's types, the engine's plan builder, removability and claims, the testing fakes).
- **Wave 2, in parallel**: 01/02 (`crates/cleaner` only), 01/03 (`crates/store`, `crates/engine/src/cleanup/history.rs`),
  01/04 (`app/ui` only) and 02/01 (`modules/local-ai-models` only).
- **Wave 3**: 02/02, then 02/03 (they need 02/01; 02/02 needs 01/01's `Removal::MoveTogether`).
- **Wave 4, in parallel**: 03/01 (case files), 03/02 (`crates/e2e`), 03/03 (`app/ui`), once wave 2 and 3 are in.
- **Wave 5**: 04.

Which files each plan touches:

| Plan | Owns                                                                                                                               | Shared files it touches                                                                                                    |
| ---- | ---------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| 01   | `crates/cleaner/src/remove/together.rs`                                                                                            | `crates/core/src/{removal,removability}.rs`, `crates/core/src/cleanup/{plan,port,outcome,view,journal}.rs`, `crates/core/locales`, `crates/engine/src/{session,claims}.rs`, `crates/engine/src/cleanup/{plan,history}.rs`, `crates/cleaner/src/{pipeline.rs,remove/mod.rs}`, `crates/cleaner/tests/never.rs`, `crates/store/src/journal/`, `crates/testing/src/`, `app/ui/src/product/confirm-dialog/`, `app/ui/src/ipc/bindings.generated.ts`, `app/ui/src/locales/`, `docs/architecture/removal.md` |
| 02   | `modules/local-ai-models/src/ollama/`, `fixtures/homes/local-ai-models-ollama-*.toml`, `fixtures/golden/local-ai-models-ollama.json`, `docs/categories/local-ai-models-ollama.md` | `modules/local-ai-models/{stores.toml,locales/en.json,Cargo.toml}`, `modules/local-ai-models/src/{lib,stores,name,categories,gathered,report}.rs`, `Cargo.lock`, `docs/categories/README.md`, `docs/dependencies.md` |
| 03   | `fixtures/sandboxes/local-ai-models-ollama{,-unreadable,-download}.toml`, their goldens, `fixtures/homes/local-ai-models-ollama-flow.toml`, `crates/e2e/tests/local_ai_models_ollama.rs`, `app/ui/src/test/ollama-scan.ts` | `fixtures/sandboxes/local-ai-models.toml` and its golden, `app/tauri/tests/sandboxes.rs`, `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/`, `docs/testing.md` |
| 04   | —                                                                                                                                  | `docs/` (architecture, restrictions, testing, manual-testing, categories, ideas, plans README)                              |

`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand: regenerate them after a merge
and run the checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI;
  `crates/cleaner` and `modules/local-ai-models` themselves at or above 90 %.
- `git diff <plan 024's head> -- crates/scanner crates/core/src/walk.rs crates/core/src/scan_cache.rs` is empty: the
  walk and the scan cache do not change.
- The cleaner's tests prove, negatives first: a together item any part of which fails a check (outside the roots,
  protected, through a link, replaced, vanished, grown, in use, mapped) moves nothing; a dry run moves nothing; a Trash
  refusal of a later part ends "only part of it went" with every moved part journalled; `tests/never.rs` holds with
  together items.
- Put back of a together item brings every part back, the subject last, and never the subject alone.
- The module's contract test passes with `Catalogue::Required`: four category files with the same dangers, every text
  and every declared left-out reason used by a fixture run, every part of every removal a matched path.
- Nothing is offered when a manifest is unreadable or over 4 KiB, a download is partial, or a store entry is not the
  user's; an incomplete model is left out; a layer another model names never leaves with a model; a model's size is its
  freed size (the fixtures' and the case files' negatives).
- `npm run sandbox:test` passes for the three new case files and every other one; it fails when a case's expectation is
  changed on purpose (once per kind, written in 03/01).
- `npm run test:e2e` passes with the core flows and the UI scenarios.
- `npm run dev:sandbox -- local-ai-models-ollama` (and `-unreadable`, `-download`): every line of `EXPECTED.md` holds
  by hand.
- On the developer's Mac (`npm run dev`, a dry run): every offered model checked against `ollama list` and the
  manifests by hand (04/02).

## Ideas and deferred items this touches

- Ollama's models (deferred): taken by this plan; the file and its row are removed when the plan
  is written.
- [Local AI models](../../ideas/local-ai-models.md): its Ollama line and its "freed bytes, not listed bytes" answered
  by this plan; other apps' models and the chunk cache stay ideas.
- [Default removal setting](../../deferred/default-removal-setting.md), [removal sandbox run](../../test-cases/sandbox-cleanup.md):
  touched (a new removal kind), not due.

## Risks

- **Ollama deletes unnamed layers at its start** (to verify): two different models sharing a base, both removed, leave
  the base layer unnamed, and Ollama deletes it outside the Trash; their put back then needs `ollama pull`.
- **A created model** looks downloaded: Medium, and the texts say how it comes back.
- **A model loaded between the snapshot and the act** moves while mapped (the snapshot is at most two seconds old).
- **A manifest over 4 KiB** blocks every Ollama offer: measured in 04/02.

## Agent's calls (summary)

The full list with costs is in [`00-design.md`](./00-design.md), "Agent's calls (to check)":

1. One finding whose removal is several paths, not a manifest finding plus layer findings.
2. Every part checked before the first moves; subject first, back last; "only part of it went" when the act stops.
3. Tags naming the same files are one finding.
4. The 4 KiB reader limit stays; a manifest over it blocks Ollama's offers.
5. Any unreadable manifest, foreign entry or partial download blocks every Ollama offer.
6. No `ollama rm`, no HTTP request: the Trash.
7. Every Ollama model is Medium (not "not claimed or High" for a created one: it cannot be told).
8. Ollama running keeps nothing; the cleaner's mapped-file check and the partial download guard instead.
9. Layers no manifest names are not claimed.
10. Three interests on the manifests' folders to read their owner.
11. The date is the newest of the manifests' change and the layers' access; no age threshold.
12. Ollama is a row of plan 023's module with a shape of its own.
13. Only the default place; `OLLAMA_MODELS` is not read.
14. The confirmation lists the other paths on demand.
15. The journal's version goes to 2.
16. No new dependency.

## Open questions

None.

## Conclusions

Built on `feat/029-ollama-models` over plan 024's head (a4db65e0), **awaiting the user's walk**: the plan is not marked
done until the user has looked at the result (the list is
[`04-finish/02-run-and-conclusions.md`](./04-finish/02-run-and-conclusions.md)). The branch is stacked on
`feat/024-unused-toolchains` and is squash-merged after it.

**What was built.** The core removes several paths as one confirmed item (`Removal::MoveTogether`): every part passes
the guard before the first moves, the subject goes first, the growth check runs once on the sum, every moved part is
journalled (journal version 2), and put back brings the other parts back first and the subject last, only when all of
them came back. The confirmation lists the other files on demand. On it, the module `local-ai-models` offers Ollama's
models (category **Ollama models**, Medium), one finding per model: the manifest and the layers no other model names,
sized by what they free, with a reason that says how large the model is in all and which model shares the rest; tags
that name the same files are one finding. Nothing of Ollama is offered when a manifest cannot be read, is over 4 KiB,
a download is in progress, or an entry of the store is not the user's; an incomplete model is left out with its reason.
Around it: seven fixture homes and a golden, three case files, eight core e2e flows, the UI scenarios and baselines,
and the docs as built. No new dependency (`serde_json` became a normal dependency of the module). The walk, scanner and
scan cache are unchanged (`git diff a4db65e0 HEAD -- crates/scanner crates/core/src/walk.rs crates/core/src/scan_cache.rs`
is empty).

**Numbers on the last code commit (ece75679), on the reference Mac.**

| Check                                   | Result                                                                          |
| --------------------------------------- | ------------------------------------------------------------------------------- |
| `npm run pre-push`                      | green: lint, guards, knip, machete, deny, script tests (78), tests with coverage |
| Rust tests and coverage                 | 3,729 passed (5 skipped); 97.5 % of lines (floor 90 %); plan 024: 3,552         |
| `crates/cleaner` alone                  | 233 tests; 98.16 % of lines (floor 90 %)                                        |
| `modules/local-ai-models` alone         | 160 tests; 99.26 % of lines (floor 90 %)                                        |
| UI unit tests and coverage              | 1,157 passed in 130 files; 98.6 % of lines (floor 90 %); plan 024: 1,150        |
| Playwright, chromium and WebKit         | 512 passed in one run, no flake; `git status` clean after it                    |
| Perf                                    | not run: the walk, scanner and cache are unchanged, the UI change is a dialog line |
| What the module offered on the Mac      | pending: the dry run is the user's                                              |
| The largest manifest on the Mac         | pending: the `wc -c` line is the user's                                         |

Playwright was run in the two browser projects (`--project chromium --project webkit`); `pre-push` does not run it.
The watched chromium baseline `category-details-app-leftovers-chosen` passed.

**What changed from the design and why.**

1. **Pre-flight rulings** (a scan of the briefs against the code found 24 findings; each took its recommended
   resolution): F1, until the cleaner arrives, every arm of the cleaner fails closed on a together item; F3, the
   per-part check and one growth check on the sum; F4 and F5, the pure Ollama parts are built without the shipped row,
   and tasks 6 and 7 (the layers, the decision, the fixtures) were built together in one commit because the contract
   test is green only with all three, which also moved plan 023's not-claimed home and sandbox case to a store without
   its manifest; F6, the shared-layer case is a positive case; F14, the confirmation's list opens in place in a plain
   group and below the virtual list in a virtual group.
2. **Cancel is read before every part**, the first included (design call 2): a cancel after the subject moved ends
   "only part of it went", journalled.
3. **The journal gained `went` and `back`.** `went` is how many parts moved, a part whose entry landed elsewhere
   counted, so a part of unknown landing cannot make the item look unmoved; `back` is the parts a put-back attempt
   brought back, so a retry skips them. The record check is typed and fails closed (the `more` list and the part
   identities must agree). Ten older e2e journal goldens were regenerated at version 2 (only the version, `more`,
   `went` and `back` lines changed).
4. **A part that landed elsewhere is counted** as having gone, in the journal and in `Partly.done`.
5. **A non-folder or hidden folder at host, namespace or model level of the manifests' tree leaves the whole store
   out** as `ollama-unreadable` (a link, a file of a plain name, a socket, a hidden or non-UTF-8 folder): it could hide a
   manifest and make a shared layer look unshared. A hidden regular file (`.DS_Store`) is ignored.
6. **The partial act has two ways**: `Failure::Partly { done, of, way: PartlyWay::Trash | Deleted }` with two texts; the
   permanent way's says the files that went were deleted and cannot be put back (the final review found the Trash text
   on a permanent delete).
7. **The claims' `losers` holds only together findings**, so a single finding that lost a part to a together finding
   that then lost is counted again (final review).
8. **The confirm dialog's form is a toggle button** with `aria-expanded`, not a disclosure primitive: the UI has none.
9. **A model with fewer than two parts, or with no known time, is not claimed**; "not the user's" covers locks and
   restricted entries; manifests are read only after the rows that need no read.
10. **The reason with a count has plural forms** and names the sharer by its first tag's title.
11. **Models are in name order, the subject is the first manifest** (`llama3:8b` before `llama3:latest`), and the mock
    and goldens say so.
12. **Tests**: a rename stands in for a removal in the restore test (the lints forbid `remove_file` there); the mapped
    case reuses the in-use fake (F11).
13. **The scan and UI timing checks were dropped** by the user's rule of 2026-11-21 (nothing the plan changes is on a
    timed path).
14. **Reviews**: tasks 1, 2, 5, 8 approved at once; tasks 3, 4, 6+7, 9, 10 needed one round (task 6+7 two); the whole
    branch was reviewed once at the end (one important finding, six minors, all fixed in the final wave).

**By-hand mutation checks of 03/01** (each reverted): the shared-layer case's danger changed from Medium to Low, and
the incomplete case's left-out items changed from 1 to 2, each failed the autotest as it should. `npm run sandbox --
all` leaves out, whole, `-ollama-download` and `-ollama-unreadable` (they write the same lone layer as the main file).

**The calls taken while building (for the user to look at).** One finding per model whose removal is several paths;
every model Medium; any unreadable manifest, foreign entry or partial download blocks every Ollama offer; no `ollama
rm` and no request: the Trash; only the default place. The full list with costs is in [`00-design.md`](./00-design.md),
"Agent's calls (to check)".

**Left open (parked, none a defect the user can hit):** a part put back and then deleted by the user counts as back
(a known limit in `docs/architecture/removal.md`); a together item's overlap with another together item may leave
both out (safe); a retry after a partial put back keeps the subject until every part is back.

**The user's walk.** Pending. The sandbox boxes, the dry run on the real Mac and its facts "to verify" are unchecked in
[`04-finish/02-run-and-conclusions.md`](./04-finish/02-run-and-conclusions.md). The facts: the largest manifest against
the 4 KiB limit; the default host and namespace in `ollama list`'s names; partial download names while `ollama pull`
runs; whether Ollama's server deletes layers no manifest names at its start (and so a model's put back after two
sharing models were removed needs `ollama pull`); what a model loaded while its files move does; hidden tag files and
non-UTF-8 tag files, which are ignored; a manifests folder the user owns but cannot list; Ollama pruning the layers a
partly moved item left behind; and manifests over 4 KiB. The texts, the table and the category doc are corrected where
the real Mac differs. This plan becomes ✔️ done when it passes.
