# 032 — Docker: design

**Status:** 🔨 awaiting the user's walk (approved by the user's delegation of 2026-11-22) · **Version:** 0.1.0 · **Date:** 2026-11-22 ·
**Modules:** core, engine, testing, sandbox, modules, e2e, app, app/ui, docs

Written on 2026-11-22 from the one-file draft and from a read of the code on main 97d9b314, read-only. Approved by the
user's delegation of 2026-11-22: build autonomously, take the recommended decisions and the agent's calls; the user
checks everything later. Where the code or a fact on the Mac forced a choice the draft did not make, the design says
so and the choice is an agent's call. Facts marked "to check" are the user's, read-only, in 06/01.

## Intent

The user's task (translated): "Look at which baseline features we lack for releasing version 0.1.0 and draft plans for
them, with our sandbox testing and architecture in mind." On this plan (2026-11-22): "Perhaps just describe: your
Docker takes this and this — here is the command to clean it if you need."

[`goal.md`](../../goal.md), section 4, promises Docker. It is often the largest single consumer on a developer's Mac,
and the most dangerous one: a volume may hold the only copy of a database. Docker also keeps its space inside one disk
file of a virtual machine, so what Finder shows, what the Mac loses and what a prune frees are three different numbers.

## Goals

1. The user sees what Docker takes, split into build cache, unused images, stopped containers and unused volumes, in
   plain English, with how much of each is unused.
2. For each part the app shows the exact command that frees it, a **Copy** button, and one sentence on what running it
   costs. The app never runs it.
3. The disk file card: the largest the file may grow to, what it takes on the disk, what is used inside, and how the
   runtime gives freed space back to the Mac.
4. Nothing of Docker's is ever a finding: never selectable, never in a cleanup, never in the "can be cleaned" total.

## Non-goals

- **Removing anything of Docker's** (decision 1): the app runs no command that changes Docker and offers no Docker item
  in a confirmation. No `ToolCommand` removal is used.
- **Starting Docker**, changing its settings, shrinking or deleting its disk file. A runtime that is not running is
  never asked (section 3).
- Runtimes without an app bundle (Colima, Lima, Podman machines) and Rancher Desktop: deferred
  ([`../../deferred/docker-other-runtimes.md`](../../deferred/docker-other-runtimes.md), written in 06/01) with what
  each needs (agent's call 2).
- One line per image or volume, dangling images apart from tagged ones (`docker system df -v`): the whole-kind numbers
  only (agent's call 1).
- A change to the storage map: the disk file is already counted once, in its folder's line (section 6).
- "Show in Finder" for the disk file: no finding, so no `reveal_finding` (deferred in the same file).

## Facts found on the developer's Mac (2026-11-22, read-only)

| Fact                                                                                       | Found                                                        |
| ------------------------------------------------------------------------------------------ | ------------------------------------------------------------ |
| `docker` on the shell's `PATH`                                                             | `/usr/local/bin/docker`, a link into a runtime app's bundle (`OrbStack.app/Contents/MacOS/xbin/docker`), itself a link to a multi-call program of that app |
| The runtime's bundle id and version                                                        | `dev.kdrag0n.MacVirt`, 2.2.5 (its `Info.plist`)              |
| Running                                                                                    | no: no process of it; its socket `~/.orbstack/run/docker.sock` absent |
| Docker's contexts (`~/.docker/contexts/meta/*/meta.json`, `~/.docker/config.json`)          | `orbstack` → `unix://~/.orbstack/run/docker.sock`; `desktop-linux` → `unix://~/.docker/run/docker.sock`; current: `orbstack` |
| Docker Desktop                                                                             | not in `/Applications`; its container `~/Library/Containers/com.docker.docker/Data` remains, `vms/0/data` empty (no `Docker.raw`); stale sockets from February 2025 in `~/.docker/run`; `/var/run/docker.sock` links there; its privileged helper still runs |
| The runtime's group container `~/Library/Group Containers/HUAQ24HBR6.dev.orbstack`          | "Operation not permitted": guarded as other apps' data       |
| `docker system df --format json`                                                           | **not run**: the CLI on `PATH` belongs to a stopped runtime whose multi-call program may start it; the design uses the format from Docker's documentation, mocked (to check) |

What follows from them:

- **A stale socket is common** (Docker Desktop's, here, nine months after it was removed): a socket alone never means a
  runtime is up.
- **The `docker` on `PATH` may belong to a runtime other than the one that is up**, and the current context may point at
  a stopped one. A call by name could ask, or start, the wrong runtime.
- **An app opened from Finder does not get the shell's `PATH`** (launchd gives `/usr/bin:/bin:/usr/sbin:/sbin`), so
  `docker` by name is "not installed" in a release build. The runtime's own CLI inside its bundle is always there.
- **Both runtimes keep their disk file in a guarded place** (`Library/Containers`, `Library/Group Containers`): the walk
  sees it only with access to other apps' data.

## What already exists

| Need                                        | What exists                                                                                     | Plan     |
| ------------------------------------------- | ----------------------------------------------------------------------------------------------- | -------- |
| Running a program with fixed arguments      | `cx.commands().run(program, args, cx.cancel())`, `CommandRunError`, the real `ProcessCommandRunner` (60 s timeout, killed on cancel) | 002, 009 |
| A module with no categories                 | `ModuleDescriptor::new(.., vec![], ..)`; the engine always runs it (`session.rs`, `unavailable`) | 002, 006 |
| Installed and running apps                  | `cx.system().installed_apps()` (`InstalledApp.path`, `bundle_id`), `installed_apps_complete()`, `running_apps()` | 009      |
| A file's sizes                              | an item match's `Match.sizes` (`logical`: the file's length; `allocated`: what it takes)         | 004      |
| What the walk saw of a place                | `cx.place_seen(interest)`: `Read`, `Absent`, `Unread`, `NotScanned`                             | 019      |
| A socket in the walk                        | an entry of kind `EntryKind::Other`, reported as itself, never read                             | 017      |
| The known folders                           | `KnownFolder::Home`, `Containers`, `GroupContainers`                                             | 009      |
| A restored scan                             | `cx.is_restored()`                                                                               | 016      |
| A table of answers for commands             | `FakeCommandRunner` (keyed by the program and its arguments joined by spaces; records calls)    | 002      |
| Sandbox access of the guarded folders       | `access = { downloads = "granted" }` in a case, `World::access`                                  | 020      |
| A stated socket in a sandbox                | `kind = "socket"` on a `[[case.file]]`, read through `FactsLens`                                  | 017      |

**What is missing**: a way for a module to say something that is not a finding. Today a module can only push findings
(selectable, counted in totals) and left-outs (counted per category). An information-only output is new (section 1),
and the sandbox has no tool answers yet ([`../../deferred/sandbox-world.md`](../../deferred/sandbox-world.md); plan 031
left them to this plan).

## Design

### 1. Insights: what a module describes and never offers (core, engine)

A new core type, `Insight` (`crates/core/src/insight.rs`): a module's description of space it does not offer.

| Field      | Type                    | What                                                                                              |
| ---------- | ----------------------- | ------------------------------------------------------------------------------------------------- |
| `identity` | kebab-case string       | unique among the module's insights of one scan (`orbstack`)                                       |
| `title`    | `Text`                  | the card's name ("OrbStack")                                                                      |
| `summary`  | `Text`                  | one sentence under the title                                                                      |
| `figures`  | `Vec<Figure>`, ≤ 4      | a label and a size each ("On your disk", 21 GB)                                                    |
| `parts`    | `Vec<InsightPart>`, ≤ 8 | each: `label`, `size`, `detail`, `effect` (what running its command costs), `danger`, `command: Option<ShownCommand>` |
| `notes`    | `Vec<Text>`, ≤ 4        | sentences under the parts                                                                         |

- `ShownCommand` is a command for the user to copy: a `ToolId` and its arguments, and the `line` the UI shows and copies
  (`docker --context orbstack builder prune --all`), built by the core. It is refused when it has no argument, when an
  argument is empty or holds a character outside `A-Z a-z 0-9 . _ / : = @ + -` (so the line never needs quoting), or
  when it forces (`ToolCommand::forces`: `-f`, `--force`, `--force=…`, `f` among short flags). Nothing ever runs it.
- `FindingSink::describe(&mut self, insight: Insight)` with a default that drops it, so every existing sink (the tests'
  `Recorded` sinks, `Vec<NewFinding>`) is unchanged. The engine's sink keeps a module's insights; a second insight with
  the same identity in one call is dropped with a warning in the log.
- `Event::ModuleFinished` gains `insights: Vec<Insight>` (in the order given), beside `leftOut`. Insights are not
  stored, not cached, not counted: a restore runs the module again and it describes again. They are never findings,
  so no total, plan, journal or storage-map number can include them.
- The UI keeps them with the module's `leftOut` (`CoreState.modules`), and filters a hidden module's out, as
  `left-out-only.tsx` does for left-outs.
- `fresher-testing`: `ModuleRun.insights`; the golden report gains `insights` when a run has any; `check_texts_used`
  counts and checks every text of an insight; two new contract problems, `ForeignInsightText` (a key outside the
  module's namespace) and `DuplicateInsight` (an identity twice in one run).

### 2. The module `docker` (`modules/docker`, `fresher-module-docker`)

Group **Development**. No category, no setting of its own (the core's `docker.enabled` switch), no declared problem
(nothing it reports is a failure or a left-out). Its data is one table, `runtimes.toml` (rule 5):

```toml
[[runtime]]
id = "docker-desktop"
name = "Docker Desktop"                 # the product's own name: the card's title
app = "com.docker.docker"               # the bundle id: installed and running are asked by it
cli = "Contents/Resources/bin/docker"   # the runtime's own CLI, relative to its app bundle
context = "desktop-linux"               # the Docker context the runtime creates
socket = { folder = "home", path = ".docker/run/docker.sock" }
disk = { folder = "containers", path = "com.docker.docker/Data/vms/0/data/Docker.raw" }
maximum = "shown"                       # the file's length is the size limit the user set

[[runtime]]
id = "orbstack"
name = "OrbStack"
app = "dev.kdrag0n.MacVirt"
cli = "Contents/MacOS/xbin/docker"
context = "orbstack"
socket = { folder = "home", path = ".orbstack/run/docker.sock" }
disk = { folder = "group-containers", path = "HUAQ24HBR6.dev.orbstack/data/data.img" }
maximum = "hidden"                      # the file's length is not a limit the user set (to check)
```

Verified here: the OrbStack row's bundle id, CLI and context and socket; Docker Desktop's context, socket and container
folder. To check (06/01): Docker Desktop's CLI path inside its bundle and `Docker.raw`'s place; OrbStack's disk file
and the meaning of its length.

- **Interests**, two per row in the table's order: the socket and the disk file, each `Pattern::Under(folder, path)`
  with `Claim::Item` (nothing is made opaque; the storage map keeps the real folders). Index `2 × row` is the socket,
  `2 × row + 1` the disk file.
- `on_match` gathers (keyed by the scan's cancel token, as `local-ai-models`' `Gathered`): a socket counts only as an
  `EntryKind::Other` entry; a disk file only as an `EntryKind::File` (a link or a folder at that path is not seen).
- `finish` decides each row in the table's order (section 3) and describes at most one insight per row, identity = the
  row's id. It checks `cx.cancel()` between rows; the CLI's `Cancelled` returns `ModuleError::Cancelled`.

### 3. When a runtime is asked, and what is said (`decide.rs`, a pure function)

For one row, the first that applies:

| #   | When                                                                                       | State       | What the card says                                                                    |
| --- | ------------------------------------------------------------------------------------------ | ----------- | ------------------------------------------------------------------------------------- |
| 1   | the installed apps cannot be listed, or the row's app is not in a partial listing           | —           | nothing (the module is silent)                                                         |
| 2   | the row's app is not installed, and its disk file was not seen                             | —           | nothing                                                                               |
| 3   | the app is not installed (the listing is complete), the disk file was seen                 | `left`      | the file's numbers; "not installed any more, but its disk file is still here"        |
| 4   | the scan is restored                                                                       | `restored`  | the file's numbers; "scan again to see what is inside"                               |
| 5   | the running apps cannot be listed                                                          | `unknown`   | the file's numbers; "could not tell whether it is running, so it was not asked"      |
| 6   | no running app has the row's bundle id, or the walk saw no socket at the row's place        | `stopped`   | the file's numbers; "start it and scan again; Fresher never starts it"                |
| 7   | otherwise                                                                                  | `ask`       | the CLI is asked (below)                                                              |

"Not installed" is taken only from a complete listing (`installed_apps_complete()`): an app a partial listing missed
may be installed, and a `left` card would then be false. The cost: a truly left-behind disk file is not described while
the listing is partial.

**The read**: program `<app bundle path>/<cli>` (the installed app's `InstalledApp.path` joined with the row's `cli`),
arguments exactly `--context <context> system df --format json`. `--context` makes the CLI ask exactly that runtime,
whatever the current context; the bundle's own CLI is there whatever the `PATH`; both the app and its socket must be
up, so a call never starts a runtime. Then:

| Answer                                                                 | State        | What the card says                                                         |
| ---------------------------------------------------------------------- | ------------ | -------------------------------------------------------------------------- |
| `Cancelled`                                                             | —            | `ModuleError::Cancelled`                                                    |
| `NotInstalled`, `Failed`, or an exit status other than 0                | `no-answer`  | the file's numbers; "did not answer; scan again once it has started"      |
| status 0, but the output is not the four kinds below                    | `unreadable` | the file's numbers; "answered in a way Fresher cannot read, so it shows no numbers rather than guess" |
| status 0 and the four kinds read                                        | `answered`   | the figures, the four parts with their commands, the notes                |

**The answer's format** (from Docker's documentation; to check on a running runtime): one JSON object per line, with
string fields `Type`, `TotalCount`, `Active`, `Size`, `Reclaimable`:

```
{"Active":"2","Reclaimable":"1.071GB (35%)","Size":"3.034GB","TotalCount":"7","Type":"Images"}
{"Active":"1","Reclaimable":"0B (0%)","Size":"1.09kB","TotalCount":"1","Type":"Containers"}
{"Active":"1","Reclaimable":"41.2MB (100%)","Size":"41.2MB","TotalCount":"2","Type":"Local Volumes"}
{"Active":"0","Reclaimable":"512MB","Size":"512MB","TotalCount":"13","Type":"Build Cache"}
```

- The four `Type`s `Images`, `Containers`, `Local Volumes`, `Build Cache`, each exactly once; another `Type` is
  ignored; a missing or repeated one, a line that is not such an object, or an empty output is unreadable.
- A size is a decimal number and a unit, `B`, `kB`, `KB`, `MB`, `GB`, `TB` or `PB`, in powers of 1000 (Docker's own
  units), rounded to whole bytes; `Reclaimable` may end with ` (N%)`, which is dropped. Anything else is unreadable.
- `TotalCount` and `Active` are whole numbers with `Active ≤ TotalCount`; `Reclaimable ≤ Size`. Otherwise unreadable.

### 4. The insight (`report.rs`)

| Part of the insight | Value                                                                                                       |
| ------------------- | ----------------------------------------------------------------------------------------------------------- |
| identity            | the row's id                                                                                                |
| title               | `docker.title` ("{runtime}") with `runtime`, the row's `name` ("Docker Desktop", "OrbStack")                 |
| summary             | `docker.summary.<state>`, with `runtime` and, when answered, `inside` and `unused`                          |
| figures             | in order, each only when known: `maximum` (the disk file's logical size; rows with `maximum = "shown"` only), `on-disk` (its allocated size), `inside` (answered: the sum of the four `Size`s) |
| parts (answered)    | in order: build cache (Safe), unused images (Low), stopped containers (Medium), unused volumes (High); each with its size (`Reclaimable`), its detail (with `count` = `TotalCount − Active`; `docker.part.<part>.none` when the size is 0), its effect, and a command when the size is above 0 |
| notes               | `docker.note.disk-unseen` when the disk file's place was not read (`Unread`, `NotScanned`); `docker.runtime.<id>.gives-back` for every state but `left`; `docker.note.never-runs` when a part has a command |

The commands (agent's call 3), each with `--context <context>` after `docker`:

| Part               | Command                         | Why this one                                                                                       |
| ------------------ | ------------------------------- | -------------------------------------------------------------------------------------------------- |
| build cache        | `docker builder prune --all`    | frees what `Reclaimable` counts (all unused build cache, not only dangling)                        |
| unused images      | `docker image prune --all`      | frees what `Reclaimable` counts (every image no container uses)                                    |
| stopped containers | `docker container prune`        | frees what `Reclaimable` counts                                                                    |
| unused volumes     | `docker volume prune`           | the safest form: since Docker 23 it removes only unused volumes without a name; named volumes, where databases usually live, stay. The effect sentence says so, and that the number counts named ones too (to check) |

None forces: Docker asks the user to confirm each in Terminal.

### 5. Texts (`modules/docker/locales/en.json`)

Plain English (rule 7); drafts, polished when built:

- `docker.enabled.label` "Docker"; `.description` "Shows what Docker keeps on your Mac and the command that frees each
  part. Fresher never removes anything of Docker's."
- `docker.title` "{runtime}" (the row's `name`); `docker.runtime.docker-desktop.gives-back` "Docker Desktop gives the
  space freed inside it back to your Mac by itself, usually within minutes."; `docker.runtime.orbstack.gives-back`
  "OrbStack gives the space freed inside it back to your Mac by itself." (both to check).
- Summaries: `docker.summary.answered` "{runtime} uses {inside} inside its disk file; {unused} of it is unused.";
  `.stopped` "{runtime} is not running, so Fresher cannot see what is inside. Start it and scan again; Fresher never
  starts it."; `.restored` "These numbers are from your last scan. Scan again to see what is inside {runtime}.";
  `.unknown` "Fresher could not tell whether {runtime} is running, so it did not ask it."; `.no-answer` "{runtime} did
  not answer. Scan again once it has fully started."; `.unreadable` "{runtime} answered in a way Fresher cannot read,
  so it shows no numbers rather than guess."; `.left` "{runtime} is not installed any more, but its disk file is still
  on your Mac."
- Figures: `docker.figure.maximum` "Largest it may grow to"; `.on-disk` "On your disk"; `.inside` "Used inside".
- Parts (label, detail with `{count}` and plural `.one`/`.other`, `.none`, effect):
  - `docker.part.build-cache.label` "Build cache"; `.detail` "Saved build steps no build uses now."; `.none`
    "Nothing unused."; `.effect` "The next builds take longer: Docker repeats those steps."
  - `docker.part.images.label` "Unused images"; `.detail` "{count} image no container uses." /
    "{count} images no container uses."; `.effect` "An image is downloaded or built again when you need it."
  - `docker.part.containers.label` "Stopped containers"; `.detail` "{count} container that is not running." / "…
    containers that are not running."; `.effect` "Anything written inside them is lost for good."
  - `docker.part.volumes.label` "Unused volumes"; `.detail` "{count} volume no container uses." / "… volumes …";
    `.effect` "A volume may hold the only copy of a database. This command removes only volumes without a name;
    named volumes stay. Docker counts both here."
- Notes: `docker.note.disk-unseen` "Fresher could not see the disk file. Allow access to other apps' data in Settings
  and scan again."; `docker.note.never-runs` "Fresher never runs these commands. Copy one into Terminal; Docker asks
  you to confirm before it removes anything."

### 6. Counted once, and the storage map

The disk file is a file in a folder of the home: the walk sums it into its folder, and the storage map shows it in the
line of the folder or of the app the engine attributes that folder to (plan 012), at its allocated size. Insights are
not findings, so they add nothing to the map, to "can be cleaned", to a category or to a plan. **The storage map does
not change** (agent's call 5); the card's figures say where the space is. The draft's "breakdown of the disk image's
line" is the card itself.

### 7. The screen (`app/ui`)

- **The scan screen**: a section **Described, not cleaned** under the left-out rows (`left-out-only.tsx`), one card per
  insight of the last scan, in module then insight order; nothing while scanning or when there is none. The card
  (`product/insight-card/`): the title, the summary, the figures, **Details**. No checkbox, no danger zone, no
  selection: it is not a category.
- **The insight screen** (`screens/insight/`, navigation `{ kind: 'insight', module, identity }`): Back, the title and
  summary, the figures, then each part: its label, its danger badge with the level's name, its size, its detail, its
  effect, and, with a command, the command line (selectable text) and **Copy**. Then the notes. Copy writes the line
  with `navigator.clipboard.writeText`, says "Copied" through the announcer; on a refusal it says "Could not copy.
  Select the command and copy it." (agent's call 7). An insight gone from the last scan returns to the scan screen.
- Every sentence on both is the core's text, except the section's heading and sentence, the Copy button and its
  results (`app/ui/src/locales/en.json`).

### 8. The sandbox

- **Tool answers** ([`sandbox-world.md`](../../deferred/sandbox-world.md), item 2, the tools part): a case states
  `[[case.tool]]` with `program` (a path relative to the sandbox, plain names, never absolute: the CLI inside a
  declared app bundle), `args` (the exact list), `status` (0 by default) and `stdout` (inline text, at most 64 KiB);
  `world.toml` lists them as `[[tool]]`. Two cases stating the same program and arguments differently are refused.
  `World::commands(sandbox)` answers exactly the stated calls and records every other call as unanswered
  (`NotInstalled`).
- **Sandbox mode never runs a program but `/usr/bin/open`**: `SandboxPorts::real` takes a runner that sends
  `/usr/bin/open` (the system interface: Show in Finder, System Settings) to the real runner and every other call to
  the world's answers, with or without stated tools. The banner adds "tools" when the world states any.
- **The autotest fails a case file when a program it does not state was run** (after the scan and after the cleanup),
  so a removal command, or any call but the read, is a test failure.
- **Other apps' data in the world's access**: `access = { other-apps-data = "granted" }` (also `denied`, `not-asked`);
  the guarded places `home/Library/Containers` and `home/Library/Group Containers` are then left to the engine's
  coverage, and the autotest turns on `core.access.other-apps-data`. Without it the disk files are never read in a
  sandbox (its standing avoid list).
- **A sparse file with written bytes**: `written = <bytes>` on a sparse `[[file]]` writes its first bytes and leaves the
  rest a hole, so a disk file shows a large maximum and a small allocated size.
- **Insight expectations**: `[[case.insight]]` with `identity`, `figures`, `parts` and `commands` (how many parts carry a
  command). An insight the module gives that no case expects fails, naming it; an expected one not given fails. The
  golden report gains `insights`; `EXPECTED.md` gains a line per expected insight.
- **Case files** (every case negative: nothing of Docker's is ever offered): `docker.toml` (a runtime not installed with
  a stale socket says nothing; a disk file at no known place says nothing; OrbStack running, answered, four parts and
  three commands, nothing run but the read), `docker-stopped.toml` (Docker Desktop closed: its file's numbers, never
  asked; OrbStack open with no socket: never asked), `docker-no-answer.toml` (Docker Desktop answering status 1;
  OrbStack answering text it cannot read), `docker-left-behind.toml` (Docker Desktop not installed, its disk file
  left). The same runtimes' paths in every file, so `npm run sandbox -- all` leaves three of them out (known; the paths
  are the runtimes', not names to vary).

### 9. Tests

Negative cases first everywhere (rule 8, [`../../testing.md`](../../testing.md)).

- **Core**: `insight.rs` (every refusal of `Insight::new` and `ShownCommand::new`, then the line and the JSON shape);
  the engine (a module's insights reach its `ModuleFinished`, a repeated identity is dropped, a failed module's insights
  given before it failed are still sent, insights add to no total).
- **Testing**: the golden's `insights`, `check_texts_used` with insight texts, `ForeignInsightText`, `DuplicateInsight`.
- **Module**: `runtimes.rs` (every refusal, the shipped table pinned), `answer.rs` (every unreadable shape, sizes and
  units; property: a size printed in Docker's units reads back within rounding), `decide.rs` (one test per row and per
  answer), `tests.rs` (on hand-built matches: the exact program and arguments, nothing asked in rows 1–6, cancel),
  fixtures and golden, the contract with `Catalogue::Required` (no category file is due).
- **Sandbox**: the world's tools (refusals, round trip, answers, unanswered calls), the access of other apps' data,
  `written`, the insight expectation (each failure), the four case files and their goldens.
- **Core e2e** (`crates/e2e/tests/docker.rs`): no finding, total or plan item of Docker's; only the read is run; a
  stopped runtime and a restored scan run nothing; the insight's parts and command lines; cancel during the read.
- **UI**: unit tests of the card, the section, the screen, Copy; the UI e2e of the section, the screen, Copy and that
  Review is unaffected; visual baselines and axe.

## Risks

- **The answer's format differs from the documentation** (Docker changes it, a runtime's CLI prints otherwise): the
  card says the answer could not be read and shows no numbers; nothing wrong is shown. 06/01 checks it on a running
  runtime.
- **A runtime's CLI starts its engine when called**: rows 6 and 7 need its app running and its socket present first.
  A runtime whose app runs and whose socket exists but whose engine is paused may wake up: that is the runtime's own
  behaviour while the user has it open (to check).
- **A copied command acts on more than the card counted** (images used by nothing now but wanted later; a named volume
  the user runs `--all` on by hand): the effect sentences say what each loses; Docker asks for confirmation; the app
  never runs them.
- **The disk file's place is guarded**: without other apps' data the card shows the inside numbers only, and the note
  says how to see the rest.
- **The UI misses a restore's events**: insights, like left-outs, arrive with `ModuleFinished`; a UI that connects after
  a restore shows none until the next scan (the same gap as left-outs, in
  [`diagnostics-screen.md`](../../deferred/diagnostics-screen.md)).
- **Rebase over plans 031, 033, 034 and 037**: they add modules (registration lines, module lists), case files, a `trash`
  section in every sandbox golden (037) and the Trash line on the scan screen (037). Mechanical: merge the lists,
  regenerate the goldens and `bindings.generated.ts`, keep both sections of the scan screen.

## Open questions

None.

## Decisions

1. **Docker is described, not cleaned** (the user, 2026-11-22): the app shows what Docker takes and the command that
   frees each part; the user runs it if they want.
2. The user's delegation of 2026-11-22 (translated): "work autonomously with subagents, take the recommended decisions;
   implement all the features; write into docs/deferred what I will need to look at." This design is approved by it.
3. No version bump: 0.1.0 ([`../README.md`](../README.md), "Versions").

## Agent's calls (to check)

Made while writing this design from the code, the rules and the Mac; each with what it costs if wrong.

1. **Whole-kind commands** (`prune`) with the four numbers of `docker system df`, not one line per object. Cost if
   wrong: a user who wants to remove one image still needs Docker's own tools.
2. **Only runtimes with an app bundle, as rows of `runtimes.toml`** (Docker Desktop, OrbStack); Colima, Lima, Podman
   and Rancher Desktop deferred. The gate needs a running app and the CLI comes from its bundle; those runtimes need
   another sign that they are up and another place for the CLI. Cost if wrong: their users see nothing yet.
3. **The commands shown, with `--context`**: `builder prune --all`, `image prune --all`, `container prune`, `volume
   prune` (the unnamed-only form). Cost if wrong: a number and its command disagree for volumes (the sentence says so),
   or a longer line to copy.
4. **No category and no category file**: the module has nothing to clean; its texts live in its own dictionary. The
   draft said "the core's dictionary"; the module's is the place every module's texts live. Cost if wrong: none.
5. **No storage-map change**: the disk file is counted once already, in its folder's line. Cost if wrong: the map
   does not say "of which build cache…" on Docker's line; the card does.
6. **Asked only when its app runs and its socket is present, and never in a restored scan** (goal 8: a restore at
   launch is not the user's action; a call may wake a runtime). Cost if wrong: the card after launch shows file numbers
   until the user scans.
7. **Copy through the web view's clipboard**, with selectable text as the fallback, not a new Tauri plugin. Cost if
   wrong: Copy fails in the built app and the user selects the text (06/01 checks it).
8. **A new core output, `Insight`, sent with `ModuleFinished`**, generic (no Docker in the core), rather than a
   Docker-only type or a category with fake findings. Cost if wrong: a wider core change than one module needs; the
   next information-only module (local snapshots, purgeable space) reuses it.
9. **Sandbox mode never runs any program but `/usr/bin/open`**, even without stated tools (today it runs real ones).
   Cost if wrong: none found; a future module that needs a real program in sandbox mode states its answer.
10. **Tool answers inline in the case file** (`stdout = "…"`), not in a file beside it as the deferred item said.
    Cost if wrong: a long answer makes a long case file (64 KiB cap).
11. **Group Development**, module id `docker`, insight identities = row ids. Cost if wrong: names to change.
12. **The danger levels of the parts** (build cache Safe, images Low, stopped containers Medium, volumes High) only
    colour the wording; nothing is selectable. Cost if wrong: a badge one level off.

## Conclusions

See [`readme.md`](./readme.md), "Conclusions": what was built, the numbers, what changed from this design and the calls taken while building.
