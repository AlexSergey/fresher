# 024 — Unused toolchain versions

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 ·
**Modules:** core, testing, sandbox, modules, e2e, app, app/ui, docs

Written on 2026-11-21 from the idea [Unused toolchain versions](../../ideas/unused-toolchain-versions.md) and from a
read of the code on plan 023's head (b8180b53). Approved by the user's delegation of 2026-11-20: after plans 027, 028
and 023, build plan 024 autonomously, take the recommended decision on every question and leave each as an agent's
call; the user checks everything together later. The design, with its agent's calls: [`00-design.md`](./00-design.md).

## Goal

Version managers never remove an old version. The module `unused-toolchains` (`fresher-module-unused-toolchains`,
group **Development**) offers, one finding per version, the **rustup toolchains** (Low) and the **nvm Node versions**
(Medium, behind the switch **Node versions (nvm)**, off by default: the user's decision of 2026-11-19) that **nothing on
the Mac points at**: not the manager's default, not rustup's overrides or nvm's aliases, not a pin file
(`rust-toolchain.toml`, `rust-toolchain`, `.nvmrc`, `.node-version`, `.tool-versions`) anywhere in the scanned folders
outside a project's `node_modules`. The default, rustup's moving channels and the newest Node version are never
offered; when a pointer cannot be read, or the walk did not read every folder, nothing of that manager is offered and
the module says why. The version's folder goes to the Trash; no tool command runs.

**Python, Java, Android, fnm, volta and asdf's or mise's own installs are not in this plan** (agent's call 1): each
needs a fact the core does not have (a link's target, every virtualenv on the Mac, a build file's code, administrator
rights). The work is [deferred](../../deferred/other-toolchain-versions.md).

**The core gains one small capability first** (plan 01): a module may read a small text file its matches showed
(`cx.system().small_file`, at most 4 KiB, never following a link) through the same reader the walk uses for
`CACHEDIR.TAG`. Every "used" signal of every manager lives in such a file, so no toolchain can be done without it. The
walk, the scanner and the scan cache do not change, so no scan bench is due. The test worlds learn file text and a case
file's settings (plan 01/02).

Branch: `feat/024-unused-toolchains`, stacked on plan 023's head, squash-merged into `main` once when done
([`../../workflow.md`](../../workflow.md)). Depends on plans [006](../006-permissions/readme.md) (needs),
[008](../008-safe-removal/readme.md), [009](../009-system-knowledge/readme.md), [011](../011-diagnostics/readme.md),
[015](../015-module-groups/readme.md), [018](../018-developer-caches/readme.md) (`Interest::holding`),
[021](../021-stale-projects/readme.md) (the projects with their artifact folders) and
[023](../023-local-ai-models/readme.md) (the shape of a store-table module).

## Plans

| #   | Plan                                          | Status      | Depends on | What                                                                                       |
| --- | --------------------------------------------- | ----------- | ---------- | ------------------------------------------------------------------------------------------ |
| 01  | [What the core lacks](./01-reader/readme.md)  | 🔨 in progress | —          | the small-file reader; file text in fixture trees and settings in case files               |
| 02  | [The module](./02-module/readme.md)           | 🔨 in progress | 01         | the crate, the table, the texts, the pointers, the decision, fixtures, registration        |
| 03  | [End to end](./03-end-to-end/readme.md)       | 🔨 in progress | 02         | the four case files, the core e2e flows, the UI                                            |
| 04  | [Finish](./04-finish/readme.md)               | 🔨 in progress | all        | the docs, the runs by hand, the conclusions                                                |

### Order and parallel work

```
01/01 ──┐
        ├── 02/01 ──┬── 02/02 ── 02/03 ── 02/04 ──┬── 03/01 ──┬── 04/01 ── 04/02
01/02 ──┘           │                              └── 03/02 ──┤
                    └── 03/03 ─────────────────────────────────┘
```

- **Wave 1, in parallel**: 01/01 (`crates/core`, `crates/testing`'s fakes, the `SystemPorts` sites, `app/tauri`'s
  compose) and 01/02 (`crates/testing`'s tree description, `crates/sandbox`, `app/tauri/tests/sandboxes.rs`). They
  share `crates/testing`: 01/01 adds `system_fakes.rs` and `small_files.rs`, 01/02 changes `tree_description.rs`;
  the second to finish rebases.
- **Wave 2**: 02/01 (the crate, its table, texts and category docs).
- **Wave 3, in parallel**: 02/02 (`modules/unused-toolchains/src/pointers/` only) and 03/03 (`app/ui` only, once
  02/01's texts exist).
- **Wave 4**: 02/03 (the decision), then 02/04 (fixtures, golden, registration).
- **Wave 5, in parallel**: 03/01 (case files, `app/tauri/tests/sandboxes.rs`) and 03/02 (`crates/e2e`).
- **Wave 6**: 04.

Which files each plan touches:

| Plan | Owns                                                                                                                    | Shared files it touches                                                                                                     |
| ---- | ----------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| 01   | `crates/core/src/system/small_file.rs`, `crates/testing/src/small_files.rs`                                             | `crates/core/src/system/{facts,mod}.rs`, `crates/core/src/lib.rs`, every `SystemPorts { .. }` site, `app/tauri/src/compose.rs`, `crates/testing/src/{tree_description,system_fakes,run_module}.rs`, `crates/sandbox/src/{case,generate,layout}.rs`, `app/tauri/tests/sandboxes.rs`, `docs/architecture/modules.md`, `docs/testing.md` |
| 02   | `modules/unused-toolchains/`, `fixtures/homes/unused-toolchains-*.toml`, `fixtures/golden/unused-toolchains.json`, `docs/categories/unused-toolchains-*.md` | `Cargo.lock`, `app/tauri/Cargo.toml`, `app/tauri/src/{compose,commands}.rs` (the module lists), `docs/categories/README.md`, `docs/dependencies.md` |
| 03   | `fixtures/sandboxes/unused-toolchains{,-unknown,-off,-documents-denied}.toml`, `fixtures/golden/sandbox-unused-toolchains*.json`, `fixtures/homes/unused-toolchains-flow.toml`, `fixtures/golden/unused-toolchains-flow.json`, `crates/e2e/tests/{unused_toolchains.rs,unused_toolchains_home/}`, `app/ui/src/test/unused-toolchains-scan.ts` | `app/tauri/tests/sandboxes.rs`, `crates/e2e/Cargo.toml`, `app/ui/e2e/{core-mock.ts,flows.spec.ts,visual.spec.ts,axe.spec.ts}`, `app/ui/e2e/screenshots/`, `docs/testing.md` |
| 04   | `docs/deferred/other-toolchain-versions.md` (checked)                                                                   | `docs/` (architecture, testing, manual-testing, categories, ideas, deferred, plans README)                                  |

`Cargo.lock`, `package-lock.json` and `bindings.generated.ts` are never merged by hand: regenerate them after a merge
and run the checks.

## Acceptance

- `npm run pre-push` passes on the branch's last commit; coverage stays at or above 90 % of lines in Rust and the UI;
  `modules/unused-toolchains` itself at or above 90 %.
- `git diff <plan 023's head> -- crates/scanner crates/cleaner crates/engine/src/session.rs` is empty: the walk, the
  scan cache and the cleaner do not change; the core's change is `small_file.rs` and the `SystemPorts` field.
- The module's contract test passes with `Catalogue::Required`: two category files with the same dangers, every text
  and every declared left-out reason used by a fixture run, every removal a matched path.
- Nothing a pointer keeps is offered: the default, an override, an alias (through a chain), a pin of each kind (in a
  guarded folder too), a moving channel, the newest Node version, a link, a folder off the shape (the fixtures' and the
  case files' negatives). A pin inside `node_modules` keeps nothing (a positive).
- Nothing of a manager is offered when its default or a pin cannot be read, or when the walk did not read every
  folder; nothing at all without access to Documents; no Node version with the switch off.
- `npm run sandbox:test` passes for the four case files; it fails when a case's expectation (a danger, a left-out's
  items) is changed on purpose (done once by hand per kind, written in 03/01).
- `npm run test:e2e` passes with the core flows and the UI scenarios.
- `npm run dev:sandbox -- unused-toolchains` (and `-unknown`, `-off`, `-documents-denied`): every line of
  `EXPECTED.md` holds by hand.
- On the developer's Mac (`npm run dev`, a dry run): every offered version checked by hand against its manager's own
  listing (04/02).

## Ideas and deferred items this touches

- [Unused toolchain versions](../../ideas/unused-toolchain-versions.md): becomes this plan for rustup and nvm; the
  rest is deferred; the open questions are answered (the Trash wins over the managers' commands; AVDs and Gradle's JDKs
  go with the deferred item).
- [Other toolchains' unused versions](../../deferred/other-toolchain-versions.md): a new deferred item, written with
  this plan.
- [Ollama's models](../../deferred/ollama-models.md): its first need (a module reads a small file) is built here; its
  removal of several paths is still missing, so it stays deferred, its "Due at" updated.
- [The world outside the disk in a sandbox](../../deferred/sandbox-world.md): touched, not due (settings in a case file
  are not the world outside the disk).
- [Developer environment cleaner](../../ideas/developer-environment-cleaner.md): touched, not changed (Xcode and IDE
  versions stay there).

## Risks

- **A pointer the module cannot see** (an editor's path, a script's `+toolchain`, an environment variable, a project
  outside the scanned folders): the version is offered; the Trash keeps it, the reason names the command that brings
  it back; Node is Medium and off by default.
- **A real Mac's walk is never complete**: every version is left out `partial-scan`; to verify in 04/02, with the
  follow-up named in the design.
- **rustup's `update-hashes`** after a removal, and **linked pins**: to verify in 04/02.

## Agent's calls (summary)

The full list with costs is in [`00-design.md`](./00-design.md), "Agent's calls (to check)":

1. Only rustup and nvm; pyenv, Java, Android, fnm, volta and asdf/mise installs are deferred.
2. A module reads a small file through a new system port in `finish`, not through the walk.
3. "Read only what your matches showed" is a rule of review, not an engine check.
4. The version folder goes to the Trash; no tool command.
5. Only the default places and the named files; no environment variable, profile or editor setting.
6. Both categories require Desktop, Documents and Downloads; a narrowed scan root is not detected.
7. Never offered: the default, rustup's moving channels, the newest Node version.
8. Conservative pins: anywhere scanned but `node_modules`, prefixes keep every match, any unreadable pointer blocks.
9. No age threshold and no day setting.
10. The version folders are claimed whole even with the Node switch off.
11. A restored scan decides from the cached matches and the files read now.
12. Node versions are Medium (global packages cannot be seen); Rust Low.
13. Versions something keeps are not reported.
14. One module, group Development, two categories, one switch for Node; Rust on by default.
15. The test worlds learn file text and case-file settings.
16. The projects' `LastChange` is not used; only their artifact folders.
17. No new dependency, no new UI component.

## Open questions

None.

## Conclusions

Built on `feat/024-unused-toolchains` over plan 023's head (8448d6cc), **awaiting the user's walk**: the plan is not
marked done until the user has looked at the result (the list is
[`04-finish/02-run-and-conclusions.md`](./04-finish/02-run-and-conclusions.md)). The branch is stacked on
`feat/023-local-ai-models` and is squash-merged after it.

**What was built.** The module `unused-toolchains` (group Development) offers rustup toolchains (Low) and nvm Node
versions (Medium, behind a switch that is off by default) that nothing keeps: not the default, an override, an alias
or its chain, a moving channel, the newest Node version, a linked or not-yours folder, nor any pin in the scanned
folders (`rust-toolchain(.toml)`, `.nvmrc`, `.node-version`, `.tool-versions`), the pins of a project's artifact
folders aside. A version the module cannot read a pointer for is kept, with its reason; each offer names the date
and the command that brings the version back. The removal is the version folder moved to the Trash. Around it: a
small-file reader in the core (4 KiB, regular files, never through a link; the walk, scanner, cache and cleaner are
unchanged), file text and settings in the test worlds, five fixture homes and a golden, four case files, the core
e2e flows and the UI cases. The only new crate edge is `fresher-sandbox` to `fresher-store` (the guard and the
architecture table say so); no new external dependency.

**Numbers on the last code commit, on the reference Mac.**

| Check                                | Result                                                                           |
| ------------------------------------ | -------------------------------------------------------------------------------- |
| `npm run pre-push`                   | green: lint, guards, knip, machete, deny, script tests (78), tests with coverage |
| Rust tests and coverage              | 3,552 passed (5 skipped); 97.45 % of lines (floor 90 %); plan 023: 3,372         |
| UI unit tests and coverage           | 1,150 passed in 130 files; 98.59 % of lines (floor 90 %)                         |
| Playwright gallery, WebKit, Chromium | 490 passed in one run, no flake; `git status` clean after it (plan 023: 467)     |
| Perf                                 | not run: no timing is due, the walk and the scanner are unchanged                |
| What the module offered on the Mac   | pending: the dry run is the user's                                               |

**What changed from the design and why.**

1. `decide` takes `Version { index, name }`: the pointers' answers are by index.
2. A row whose switch is off is decided with empty pointers (no nvm file read) instead of skipped, so the rule itself
   leaves the Node versions alone and `Scan::switch_on` stays meaningful.
3. Linked version folders count as installed when the newest Node version is found, as nvm's own view does; a plain
   file named like a version does not (final fix wave, a test that failed first).
4. A nvm prefix matching nothing installed keeps nothing and reads as known; a file with a byte order mark no longer
   drops its pin (review fix: it would have failed open).
5. No `-off` home: the switch-off run reuses the offered home with the switch false; a `-kept-unknown` home was added
   for the unknown-pointer reasons.
6. `partial-scan` is proven in `src/tests.rs` only: a fixture run cannot make a walk incomplete.
7. The app's `every_registered_module_has_a_case_file` was red between 02/04 and 03/01 (no push in between).
8. The case files' main file grants Desktop, Documents and Downloads: with no grant the module does not read a pin in
   `Documents` and offers a pinned version.
9. `npm run sandbox -- all` leaves out, whole, a case file that writes a path an earlier one wrote (files ordered by
   stem, a base before its variants), naming it on stderr and in `EXPECTED.md`; the main file stays, the three
   variants run by name.
10. The core e2e turns the Node switch on with `update_setting` (the home fixture holds no settings); the restored-scan
    test is `a_restored_scan_reads_the_pins_again` and proves the re-read by changing a matched pin between restart
    and restore.
11. The UI lives on plan 027's Scan and Details screens, not on module pages; the mock leaves a Node version out as
    not-yours, since the core cannot offer one and leave another out for an unreadable pin.
12. The test reader opens files with `O_NOFOLLOW | O_NONBLOCK` and checks device and inode; the race has no test.
13. The step's scan-time check (2.9) was dropped by the user's rule of 2026-11-21.

**The calls taken while building (for the user to look at).** Only rustup and nvm in 0.1, other managers deferred;
Node versions are Medium because global packages cannot be seen; a pin anywhere counts and any unreadable pin blocks
that manager's offers; no age threshold; the version folders are claimed whole even with the switch off. The full list
with costs is in [`00-design.md`](./00-design.md), "Agent's calls (to check)".

**By-hand mutation checks of 03/01** (each reverted): a Node case at Low, two socket items, and no `[settings]` line
each failed the autotest as they should.

**Left open (parked minors, none a defect the user can hit):** two private `NoFiles` fakes in the core's facts; the
`alias_key` count of parts; a silent drop in the report's `tell` (unreachable); cloned descriptors in the report; the
all-only stderr branch of the sandbox binary untested; a duplicate of an offered test in the node_modules negative; no
"would be offered without its keeper" check in the not-claimed home; `.tool-versions` key check hard-coded.

**The user's walk.** Pending. The sandbox boxes, the dry run on the real Mac and its facts "to verify" (is the walk
complete on this Mac, a removed toolchain coming back with `rustup toolchain install` while `update-hashes` remains,
how many linked pins and whether they block nvm's offers) are unchecked in
[`04-finish/02-run-and-conclusions.md`](./04-finish/02-run-and-conclusions.md); the table, texts and category docs
are corrected where the real Mac differs. This plan becomes ✔️ done when it passes. The user checks plans 027, 028,
023 and 024 together (the delegation of 2026-11-20).

