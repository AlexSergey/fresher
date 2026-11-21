# 029 — Ollama's models: design

**Status:** ✅ approved (the user's choice of 2026-11-21) · **Version:** 0.1.0 · **Date:** 2026-11-21 ·
**Modules:** core, engine, cleaner, store, testing, modules, sandbox, e2e, app, app/ui, docs

Written on 2026-11-21 from the deferred item "Ollama's models" (its file removed by this plan), the idea
[Local AI models](../../ideas/local-ai-models.md) and a read of the code on plan 024's design
commit (39ec314f) (its small-file reader and its test-world text), read-only. Approved by the user's choice of
2026-11-21: build Ollama's models after plan 024, autonomously; take the recommended decision on every question and
record each as an agent's call; the user checks everything later. Where the code or the rules forced a different choice
than the deferred item, the design says so and the choice is an agent's call. Facts marked "to verify" are checked by
hand in 04/02.

## Intent

Ollama is the most common way to run a language model on a Mac, and its models are the largest files in a developer's
home: several gigabytes each, pulled to try and then forgotten. Plan 023 offered the models of Hugging Face and LM
Studio, one finding per model, and deferred Ollama because one Ollama model is not one folder: a small manifest names
the layer files the model uses, the layers live in one shared folder, and models share layers. The idea asks for one
finding per model with the size it really frees ("14 GB, of which 9 GB is freed; the rest is shared with
`llama3:70b`"), and never a model that cannot come back without saying so.

## Goals

1. **The core removes one confirmed item made of several paths** (plan 01): a finding's removal may name up to 64
   paths that go together, the first being its subject. Every path is checked by the guard before the first one moves,
   each is journalled, the item's sizes are what it frees, and put back brings the paths back together, the subject
   last. The confirmation lists the other paths on demand.
2. **The module `local-ai-models` offers Ollama's models** (plan 02), one finding per model of its default store
   (`~/.ollama/models`), in a new category `local-ai-models.ollama` (Medium): the model's manifest (or the manifests of
   its tags when several tags name the same layers) and the layers no other manifest names go to the Trash together.
3. **Honest sizes**: the finding's size is what it frees; its reason says how large the model is in all, how much of
   it is freed and which model keeps the rest.
4. **Nothing is offered when the store cannot be read whole**: an unreadable manifest, a download in progress, an
   entry of the store that is not the user's: the module offers nothing of Ollama and says why.

## Non-goals

- **Ollama's own command or its HTTP interface**: `ollama rm` and `ollama list` need its server running, a tool never
  runs in a sandbox (no case file could prove it), and its HTTP interface is a network request (privacy). The files go
  to the Trash (agent's call 6).
- **Layers no manifest names** (left by a removed model or an abandoned download): not claimed; Ollama removes them
  itself the next time its server starts (to verify) (agent's call 9).
- **Telling a model made on this Mac from a downloaded one**: nothing on the disk says which a manifest is, so every
  model is Medium and its texts say what a model made with `ollama create` needs to come back (agent's call 7).
- **A store moved by `OLLAMA_MODELS`**: only the default place, as plan 018 decided for every cache (agent's call 13).
- **A removal of several paths for other modules**: the capability is general, but only this module uses it in this
  plan.
- **Changing the walk, the scanner or the scan cache**: the module reads manifests through plan 024's `small_file` in
  `finish`; the new interests are module data. No scan bench is due ([`../../testing.md`](../../testing.md),
  "Performance").

## What already exists

| Need                                     | What exists                                                                                                  | Plan        |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------------------ | ----------- |
| Reading a manifest                       | `cx.system().small_file(folder, name)`: a regular file of at most 4 KiB, read in `finish`, never a link      | 024         |
| A manifest's text in a test world        | `[[file]] text = "..."` in fixture trees and case files                                                      | 024         |
| The store's places                       | `Pattern::Under(KnownFolder::Home, "…/*")`, `Claim::Item`, `Interest::holding(names)`, `Match.facts`        | 002, 018    |
| One item, several things in the Trash    | `ItemDone.trashed` / `ItemRecord.trashed: Vec<TrashedItem>` (an emptied folder's entries); put back goes entry by entry, only those still in the Trash | 008 |
| A file a running program maps            | the cleaner's check 10: `MacOpenFiles` lists every descriptor, working folder and **mapped file** of the user's processes | 009 |
| The module                               | `modules/local-ai-models`: its stores table, gathered matches, a pure decision, its report                    | 023         |
| A versioned journal file                 | `fresher-store`'s journal: a header with its version, one migration per version step                          | 008         |

## What is missing

1. **A removal of several paths as one item.** `Removal` names one path (`MoveToTrash`, `EmptyFolder`) or one tool
   command; a `PlanItem` has one action and one identity; the engine's removability, plan builder and claims read one
   path; the journal's begin record keeps one path per item.
2. **Ollama in the module**: its store row, the manifest reader, the layers each model uses, and the decision.

### Why one item of several paths, and not a finding per manifest plus findings per layer

The deferred item named two ways. Weighed on the rules:

| Question                        | One finding, several paths (chosen)                                                       | A manifest finding, plus a finding per unshared layer                                                     |
| ------------------------------- | ----------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| Safety (rule 1)                 | every path checked before the first moves: a model the server has loaded moves nothing   | the user can select a layer without its manifest: Ollama still lists a model whose layer is in the Trash |
| Trash first                     | everything that leaves goes to the Trash, and put back restores the model                 | moving a manifest alone leaves its layers unnamed; Ollama deletes those itself at its next start, outside the Trash (to verify), so putting the manifest back gives a broken model |
| Honest sizes                    | the model's finding is what it frees                                                       | the model's finding frees about 1 KB; its gigabytes are separate rows the user must find                   |
| Selection and Review            | unchanged: one finding, one row                                                            | needs a "goes with" link between findings that `CleanupSelection` does not have                           |
| Partial failure                 | a part refused at the act stops the item; what moved is journalled and can be put back    | each finding fails on its own: a half model with no record that the halves belonged together              |
| Core surface                    | a new `Removal` variant and its paths through the engine, the cleaner and the journal      | none in the core, much more in the module and the UI                                                       |

One item of several paths costs a core change but is the only way that keeps the Trash's promise and gives the user
one honest row per model (agent's call 1).

## Design

### 1. Several paths as one item: the core (plan 01/01)

```rust
// crates/core/src/removal.rs

/// The most paths one item may name.
pub const MAX_PARTS: usize = 64;

pub enum Removal {
    MoveToTrash(PathBuf),
    EmptyFolder(PathBuf),
    ToolCommand(ToolCommand),
    /// Several files or folders that leave together, as one item the user confirms: the first is the finding's
    /// subject and goes first; put back brings it back last.
    MoveTogether(Parts),
}

/// The paths of a `MoveTogether`: two to `MAX_PARTS`, each absolute and plain, none twice, none inside another.
pub struct Parts(Vec<PathBuf>);

impl Parts {
    pub fn new(paths: Vec<PathBuf>) -> Result<Self, RemovalError>; // TooFewParts, TooManyParts, NotPlain, PartTwice, PartInsidePart
    pub fn subject(&self) -> &Path;
    pub fn all(&self) -> &[PathBuf];
    pub fn rest(&self) -> &[PathBuf];
}
```

- It serializes as `{ "kind": "move-together", "value": ["/…", "/…"] }`.
- **Removability** (`Removability::of_removal`): it takes the facts of every path a removal names (a closure over the
  scan's matched facts). A `MoveTogether` is the removability of its first part that is not removable, in part order;
  a part no match of the scan reported is `NeedsAdministrator` (fail closed, as a single path is today).
- **The plan** (`crates/core/src/cleanup/plan.rs`): `Action::RemoveTogether(Parts)`. Its `kind()` is
  `ActionKind::Remove` (it goes the way a moved item goes: to the Trash, or deleted with the permanent choice), `path()`
  is the subject, and a new `parts()` answers every path the action acts on (`[path]` for `Remove` and `Empty`, all
  parts, none for a tool). A rehearsal of it answers `Rehearsal::PassedTogether(Vec<ItemIdentity>)`, one per part in
  order; `PlanItem::with_part_identities` keeps them (`identity()` is the subject's, `part_identities()` all of them,
  empty for any other action).
- **The cleaner's fail-closed arms** (as built, plan 01/01): until 01/02 replaced them, rehearse, act and verify
  answered `Skipped(CheckFailed)` for a together item, never the `Remove` arm; a plan whose together item does not
  carry one identity per part is refused at `freeze` (`EngineError::Rehearsal`).
- **The confirmation** (`PlanItemView`): gains `more: Vec<PathBuf>`, the parts after the subject (empty otherwise).
  The item count, the size and the highest danger the user confirms are the items', unchanged: one model is one item.
- **A partial act** (`Failure::Partly { done: u32, of: u32, way: PartlyWay }`; `PartlyWay` is `Trash` or `Deleted`,
  after the plan's method). The Trash's text is `core.cleanup.failed.partly`, "Only {done} of its {of} files went;
  the rest stayed in place. What went is in the Trash."; a permanent delete's is `core.cleanup.failed.partly-deleted`,
  "... The files that went were deleted and cannot be put back." `done` counts every part that left its place.
- **The engine** (plan 01/01):
  - `session.rs`: a finding's removability from every part's facts.
  - `cleanup/plan.rs`: `refused` accepts `MoveTogether` when its subject is the finding's `Subject::Path`; `sort_out`
    registers every part as a path that may cover another item: a single item inside a part, or of the same path, is
    `Covered`; a together item any part of which is covered by another kept item, or ignored, is left out whole
    (`Covered`, `Ignored`): nothing of an item is removed unless all of it can be. `action_order` puts it with the
    moved items.
  - `claims.rs`: a together finding claims each part, so the storage map shows the layers as explained; it counts its
    sizes when it owns every part, nothing otherwise (no two findings of this plan share a path).
  - `fresher-testing`: `FakeCleaner` answers `PassedTogether` and records every part; `assert_module_contract` checks
    that every part of every removal is a matched path.

### 2. Several paths as one item: the cleaner (plan 01/02)

The cleaner's rules hold for every part: each part is a path the guard checks (checks 1 to 10 per part, each against its
own confirmed identity; growth, check 11, once on the sum of the parts' measured sizes against the item's confirmed
size, before the first part moves), opened beneath its root through no link and held open until its act.

- **Rehearse**: every part through the guard, in order; the first part refused skips the item with that part's reason;
  else `PassedTogether` with every identity.
- **Act** (`act_on`, a new `remove/together.rs` called only from `pipeline.rs`):
  1. Verify every part with `Depth::Full`, in order (`verify_together`), each against its confirmed identity, keeping
     every `Verified` (each holds its handle). The first refusal skips the whole item with that reason, and nothing has moved: **a
     model the Ollama server has mapped is refused by check 10 on its layer before its manifest moves.**
  2. A dry run ends the item `DryRun` here.
  3. Act on the parts in order, the subject first, each with its own `Verified` and the plan's way (`to_trash`, or
     `delete` through the single deletion point with the permanent consent); the cancel token is read before every
     part, the first included (the rule of [`removal.md`](../../architecture/removal.md), "Adding a kind of
     removal"): a cancel after the subject moved ends `Partly`, journalled.
  4. Every part gone: `Moved` (or `Deleted`). A part refused, failed or cancelled after an earlier one went: the item
     ends `Failed { Partly }` and stops; `trashed` lists every part that went, so it is journalled and can be put
     back. The first part failing is that part's own failure, as a single item's.
- Why the subject first: the item stops being what its program lists (Ollama reads its manifests) in one step; a later
  part left behind is a file nothing names any more, never a model with a missing layer.
- `tests/never.rs`: its generator gains together items of real and made-up paths; the property holds: only plan parts
  (or what is inside them) leave, protected and outside trees list as before; a together item either moves every part
  or none, unless the Trash refuses a part at its act (the fake's injected refusal).
- The threat model of [`removal.md`](../../architecture/removal.md) gains a row: "an item of several paths half done":
  every part verified before the first act, the subject first, every moved part journalled, put back subject last;
  left: a part the Trash refuses after earlier parts moved (reported "only part of it went", put back available).

### 3. The journal and put back (plan 01/03)

- `JournalItem` gains `more: Vec<JournalPart { path: PathBuf, identity: Option<ItemIdentity> }>`: the parts after the
  subject, written in the begin record before the first item (every part is recorded, with its identity when the plan
  had one), so a crash in the middle of a model leaves a record of every path it was about to move. The item's record
  gains `went`: how many parts went, a part that left its place but landed elsewhere included. A put-back record gains
  `back`: the parts that attempt brought back, by their place among the item's parts (the subject is 0). The journal's
  file is version 2; the migration from version 1 adds `more: []` to every begin item, `went: null` to every item
  record and `back: []` to every put-back record; an older build reads a version-2 file as `NewerVersion`
  (read-only), as today.
- Put back of an item with `more` (the engine's `history.rs`, `put_back_together`): the parts other than the subject
  that went and are not back yet first, in the reverse of the order they moved; **the subject only when every other
  part that went is back** (brought back now, or by an earlier attempt, which `back` records); otherwise the subject
  stays in the Trash and the outcome is the first refusal. A part that went with no known landing, or whose entry is
  no longer in the Trash and was never brought back, keeps the subject there. A record that does not hold together
  (more parts went than the item has, an entry of a path that is none of its parts) is checked in a typed way and
  puts nothing back. A model never comes back as a manifest that names a layer still in the Trash. Known limit: a part
  put back and then deleted by the user counts as back (the journal records what was brought back, not what the disk
  holds later); the subject may then come back without it.
- History shows one row per item, as today; its put back is the whole model.

### 4. The confirmation (plan 01/04)

`ConfirmDialog`'s item row: when `more` is not empty, a button under the path, "and {count} more files that go with
it" (`ui.confirm.more-files.one` / `.other`), with `aria-expanded` and `aria-controls`, which opens the list of their
paths on demand (rule 7: detail on demand). In a plain group the list expands in place under the row; in a virtual
group (more than 50 items) the row shows the button and the list opens below the virtual list, named by
`ui.confirm.more-list`. The selection, Review and the summary are
unchanged. `bindings.generated.ts` is regenerated. The row is not the scan screen, a virtual list or motion: no
performance run ([`../../testing.md`](../../testing.md), "Performance").

### 5. The store (`stores.toml`, plan 02/01)

One row more, after the three of plan 023:

```toml
[[store]]
id = "ollama"
tool = "ollama"
folder = "home"
path = ".ollama/models"
shape = "ollama-store"
category = "local-ai-models.ollama"
```

- The shape `ollama-store` takes a path with no `*` (the store folder itself), no `sign`, no `app` and one category;
  the table refuses any other combination, one error each, as it refuses the other shapes' (a test per refusal).
- The shape knows Ollama's layout as `hub-repo` knows its `--` split: the store holds `manifests/` and `blobs/`; a
  manifest is `manifests/<host>/<namespace>/<model>/<tag>`; a layer is `blobs/sha256-<64 hex>`.
- **Interests** of the row (all `Claim::Item`, so the storage map keeps the real folders):

| #   | Pattern under `~`                       | What it gives                                                     |
| --- | --------------------------------------- | ----------------------------------------------------------------- |
| a   | `.ollama/models`, holding `manifests`, `blobs` | the store is there and has its layout                      |
| b   | `.ollama/models/manifests/*`            | the hosts' folders: their owner                                   |
| c   | `.ollama/models/manifests/*/*`          | the namespaces' folders: their owner                              |
| d   | `.ollama/models/manifests/*/*/*`        | the models' folders: their owner                                  |
| e   | `.ollama/models/manifests/*/*/*/*`      | the manifests (one file per tag): path, sizes, times, facts      |
| f   | `.ollama/models/blobs/*`                | the layers and any partial download: path, sizes, times, facts    |

- Interests b to d exist so that a folder of the manifests' tree the user does not own (made by `sudo ollama …`), whose
  manifests the walk may not list, makes the store not the user's (section 7, row 2) rather than hiding a manifest
  that names shared layers (agent's call 10).

### 6. Manifests (`ollama/manifest.rs`, plan 02/01)

- Read in `finish` through `cx.system().small_file(model folder, tag)`, for every match of interest e that is a file:
  a file the module's own match showed (024's rule of use). `Text` is parsed; `Missing`, `NotAFile` (a link),
  `TooLong`, `NotText` and `Unreadable` make the manifest unreadable.
- Parsed with `serde_json` (unknown fields allowed: Ollama adds fields): `schemaVersion` must be 2; `config` and every
  entry of `layers` (at least one) need a `digest` of the shape `sha256:<64 lowercase hex>` and a `size`. The digest
  names the file `blobs/sha256-<hex>`. Anything else is unreadable. A digest listed twice counts once.
- **Size**: an Ollama manifest is a few hundred bytes per layer (about 1 KB for a model of five layers). A manifest over
  4 KiB (about twenty layers) is `TooLong`, so unreadable, so nothing of Ollama is offered and the left-out says why
  (agent's call 4; the largest manifest on the developer's Mac is measured in 04/02, to verify).
- **Names** (`name.rs`, a new arm): the four components of the manifest's path, each valid UTF-8 and not starting with
  `.`; the title is Ollama's own: `model:tag` for the host `registry.ollama.ai` and the namespace `library`,
  `namespace/model:tag` for that host and another namespace, `host/namespace/model:tag` otherwise. A path off the
  shape is not a model.
- A file of interest e that is not a regular file, a folder at the tag's depth, and anything at another depth are not
  manifests: Ollama reads only `manifests/*/*/*/*` files (to verify). A link there is read as unreadable (small_file
  never follows it), so it blocks the store.

### 7. Layers and the decision (`ollama/layers.rs`, `ollama/decide.rs`, plan 02/02)

**Models.** Manifests that name exactly the same set of digests (config and layers) are one model with several tags
(`llama3:latest` and `llama3:8b` are commonly the same files): one finding, whose title names every tag in name order
(agent's call 3). Every other manifest is a model of its own.

**Freed and listed.** For a model `M`: its *listed* size is the sum of the logical sizes of every layer it names (what
`ollama pull` downloads); its *parts* are its manifests and the layers no manifest outside `M` names; its *freed* sizes
are the sum of the parts' sizes (logical, allocated, reclaimable, with their completeness). The other models that name
a layer of `M` are its *sharers*.

**The store's rows**, in order, decided once per scan; the first that applies decides for every model:

| #   | When                                                                                                      | Verdict                                             |
| --- | --------------------------------------------------------------------------------------------------------- | --------------------------------------------------- |
| 1   | no match of interest a, or it does not hold both `manifests` and `blobs`                                  | not claimed                                         |
| 2   | an entry the store's interests matched (a folder of b to d, a manifest, a layer, a partial download) is not the user's: another owner, a lock or restricted | left out `not-yours`, the store's sizes             |
| 3   | `blobs/` holds a partial download (`sha256-<hex>-partial`, or that name followed by `-<n>`)               | left out `ollama-download`, the store's sizes       |
| 4   | an entry at a host's, namespace's or model's depth of `manifests/` is anything but a folder of a plain name (a link, a file of a plain name, a folder whose name starts with a dot or is not UTF-8; a hidden file such as `.DS_Store` is ignored), or a manifest is unreadable (section 6) | left out `ollama-unreadable`, the store's sizes     |

Rows 1 to 3 and the first half of row 4 (the entries) are decided **before any manifest is read**; manifests are read
only when the store holds `manifests` and `blobs` and those rows let it through, so a store that is left out reads
nothing.

**A model's rows**, in order, once the store passed:

| #   | When                                                                                                      | Verdict                                     |
| --- | --------------------------------------------------------------------------------------------------------- | ------------------------------------------- |
| 5   | a layer it names is not among the matches, is not a regular file, or its logical size differs from the manifest's | left out `ollama-incomplete`, its parts' sizes |
| 6   | more than `MAX_PARTS` parts, fewer than two (one tag whose every layer is shared), or no time known (no manifest or layer time to date it) | not claimed                                 |
| 7   | otherwise                                                                                                 | offered, Medium                             |

- An incomplete model's layers still count as named: they are never another model's parts.
- Blob names off the shape (`.DS_Store`, a name without `sha256-`) are neither layers nor partial downloads: not
  claimed, not counted.
- **The server**: no row keeps Ollama's models while its app or server runs (agent's call 8). The server maps the
  layers of a loaded model, so the cleaner's check 10 sees the mapped file and skips the whole model before anything
  moves (section 2). A pull in progress writes a partial file, so row 3 offers nothing then.
- A left-out store is one line with the sizes of its manifests, layers and partial downloads; an incomplete model is
  one line with the sizes of its manifests and the layers it would free.
- The verdict does not depend on time or on the order of the matches; a restored scan decides from the cached matches
  and the manifests read now, so it offers the same while nothing changed.

### 8. Findings (plan 02/02)

- Identity `ollama/<host>/<namespace>/<model>/<tag>` of the model's first manifest by name order (its tags in name
  order; the subject is that manifest).
- Subject `Subject::Path` of that manifest; removal `Removal::MoveTogether` of its parts: the manifests in name order,
  then the unshared layers by name.
- Sizes: the freed sizes. Category `local-ai-models.ollama`, Medium.
- Title `local-ai-models.title` with `name`: the tags' titles joined by ", ".
- Reason, with `date` the newest of the manifests' `modified` and the parts' layers' `accessed` (`TextArg::Date`,
  "last opened or changed"), `size` the freed logical size, `name` the first tag's title:
  - no sharer: `local-ai-models.reason.ollama`;
  - one sharer: `local-ai-models.reason.ollama-shared` with `listed` (`TextArg::Size`) and `other` (the sharer's title);
  - several: `local-ai-models.reason.ollama-shared-more` with `listed`, `other` (the first sharer by name) and `count`
    (`TextArg::Count`, the others), plural by that count (`.one`, `.other`). The shared reasons name no model of their
    own: the `name` argument is only in `local-ai-models.reason.ollama`.
- Left-outs: `LeftOutItem { category: local-ai-models.ollama, reason: local-ai-models.left-out.<reason>, sizes }`.

### 9. Texts (`locales/en.json`, drafts polished when built)

- `local-ai-models.ollama.name` "Ollama models"; `.what` "The language models Ollama keeps to run them on this Mac:
  for each model a small list of its files and the layer files it names. Models can share layers."; `.made-by`
  "Ollama, an app and a command that run language models locally."; `.if-removed` "The model's list and the layers no
  other model uses go to the Trash together, and Ollama no longer lists the model. Layers another model uses stay, so
  your other models keep working."; `.comes-back` "If you downloaded it, yes: `ollama pull` with its name downloads it
  again. A model you made yourself with `ollama create` comes back only from its Modelfile and your files, or from the
  Trash."
- `local-ai-models.reason.ollama` "Kept by Ollama; last opened or changed on {date}. `ollama pull {name}` downloads
  its {size} again; a model you made yourself does not come back that way."
- `local-ai-models.reason.ollama-shared` "Kept by Ollama; last opened or changed on {date}. It is {listed} in all, of
  which {size} is freed; the rest is shared with {other} and stays."
- `local-ai-models.reason.ollama-shared-more.one` / `.other` "Kept by Ollama; last opened or changed on {date}. It is
  {listed} in all, of which {size} is freed; the rest is shared with {other} and {count} more model(s) and stays."
- Left-outs: `local-ai-models.left-out.ollama-unreadable` "One of Ollama's model lists could not be read, so it cannot
  be told which files its models share. Nothing of Ollama is offered."; `.ollama-download` "Ollama is downloading a
  model, or a download was left unfinished. Let it finish, then scan again."; `.ollama-incomplete` "Some of this
  model's files are missing or still being written. Pulling it again in Ollama repairs it."; `.not-yours` (plan 023's
  sentence, checked to read right for a store).
- The switch's description: "Finds models downloaded by Hugging Face, LM Studio and Ollama. Each one goes to the Trash;
  most come back by download."
- Core: `core.cleanup.failed.partly` (section 1). UI: `ui.confirm.more-files.one` / `.other` ("and {count} more file
  that goes with it" / "files that go with it"), `ui.confirm.more-files-of` and `ui.confirm.more-list`
  (`app/ui/src/locales`).

### 10. Tests

Negatives first everywhere (rule 8, [`../../testing.md`](../../testing.md)); the core plan's tests are written to the
cleaner's bar (every refusal before any act, races, the never property).

- **Core** (`removal.rs`, `removability.rs`, `cleanup/plan.rs`, `outcome.rs`, `view.rs`): every refusal of `Parts`; a
  together removability from the first non-removable part, a part without facts; serialization.
- **Engine** (`cleanup/plan.rs`, `session.rs`, `claims.rs`, `history.rs`): a subject that is not the first part is
  `NotTheSubject`; a covered or ignored part leaves the whole item; a single item inside a part is `Covered`; put back
  puts the subject last and keeps it in the Trash when another part cannot come back.
- **Cleaner** (`remove/together.rs`, `pipeline.rs`, `tests/never.rs`): a part outside the roots, protected, through a
  link, replaced, vanished, grown, in use (open, and mapped) each skips the whole item with nothing moved; the Trash
  refusing the second part ends `Partly` with the subject journalled; a cancel between parts; a dry run moves nothing;
  the permanent way deletes each part through the deletion point; every part's identity in the rehearsal.
- **Store**: a version-1 journal reads with `more: []`; a version-2 round trip with `more`; an unknown version stays
  read-only.
- **Module**: `stores.rs` (the new shape's refusals), `ollama/manifest.rs` (each refusal, the 4 KiB edge),
  `name.rs` (Ollama's titles), `ollama/layers.rs` (aliases, sharers, freed and listed; properties: the parts of two
  models never overlap; the freed sizes of all models plus the shared layers equal the store's layers; the result never
  depends on the order of matches), `ollama/decide.rs` (one test per row, keep before offer).
- **Fixtures**: `fixtures/homes/local-ai-models-ollama-*.toml` (`-not-claimed`, `-no-blobs`, `-kept`, `-hidden`,
  `-unreadable`, `-long`, `-offered`, and the flow home `-flow`) with the goldens. Plan 023's not-claimed home and its
  sandbox case lose only the manifest (an Ollama folder without `manifests/` stays not claimed, row 1); its golden
  stays as designed.
- **Contract**: `Catalogue::Required` with four category files; every text and left-out used; every part a match.
- **Sandbox**: `fixtures/sandboxes/local-ai-models-ollama.toml`, `-ollama-unreadable.toml`, `-ollama-download.toml`
  (a store-wide left-out blocks every case of its file, so each has its own file); plan 023's Ollama negative case
  stays in `local-ai-models.toml` without the manifest. The store-wide files write the lone layer the first file
  writes, so in `npm run sandbox -- all` they are left out whole (a path an earlier file wrote) and named in the
  checklist, and are written by name.
- **Core e2e** (`crates/e2e/tests/local_ai_models_ollama.rs`): the scan offers each model with its freed size; a
  cleanup moves the manifest and its unshared layers to the sandbox's Trash and leaves the shared layer and the sharer's
  manifest; a mapped layer and a layer replaced after the plan each skip the whole model; the journal holds every part
  of each item (`more`, `went`), put back restores every file byte for byte with the subject last (each put-back
  record's `back` in reverse order); a restore with a manifest moved away leaves the store out `ollama-unreadable`; a
  restored scan offers the same.
- **UI e2e**: the Ollama finding and its reason (the UI mock declares four categories, as the core always does), the
  confirm dialog's "more files" toggle and list (a plain group and a virtual one), a cleanup and put back; the toggle
  is covered by axe.

## Risks

- **Ollama deletes layers no manifest names when its server starts** (to verify). Removing two models that share a layer
  leaves that layer unnamed (neither model's parts include it); Ollama then deletes it at its next start, outside the
  Trash. Putting both models back after that gives broken models until `ollama pull`. The reason of each says the rest
  "is shared with … and stays", and the category's text says how a model comes back. Aliases (the commonest sharing)
  are one finding, so this needs two different models that share a base.
- **A model made with `ollama create`** looks like a downloaded one: Medium, both texts say how it comes back, the Trash
  keeps it.
- **A loaded model**: check 10 sees the mapped layer (refreshed every two seconds, the mapped regions reused for 30 s);
  a model loaded in the window after the snapshot moves, and the server keeps its mapping until it unloads it (to
  verify what Ollama answers then).
- **A folder of the store the walk could not read although the user owns it** (a broken disk): a manifest there is
  unseen and a layer it shares looks unshared; the model offered moves it, recoverable from the Trash. Rows 2 and 4
  cover the cases a person makes (`sudo`, links).
- **A manifest over 4 KiB** blocks every Ollama offer (row 4) until the reader's limit is raised for it (measured in
  04/02).
- **Ollama's facts** (the default host and namespace in titles, `manifests/*/*/*/*` as the only place Ollama reads,
  partial download names, prune at start) are to verify on a Mac with Ollama in 04/02.

## Open questions

None.

## Decisions

- The user's choice of 2026-11-21 (translated): "after plan 024, build Ollama's models, autonomously; I check
  everything later." This design is approved by it.
- No version bump: 0.1.0 ([`../README.md`](../README.md), "Versions").

## Agent's calls (to check)

Made while writing this design from the code and the rules; each with what it costs if wrong.

1. **One finding whose removal is several paths, not a manifest finding plus layer findings** (section "Why one item
   …"): the only way that keeps everything in the Trash, honest sizes and one row per model. Cost if wrong: a core
   capability (a `Removal` variant, its paths through the engine, the cleaner, the journal) that only this module uses.
2. **Every part is checked before the first moves; the subject goes first and comes back last; a part failing after
   another went ends the item "only part of it went"**, with everything moved journalled. Cost if wrong: a cancel or a
   Trash refusal in the middle leaves a model without its list (its layers nameless), which put back repairs.
3. **Tags naming exactly the same files are one finding**: Ollama shows them as rows with the same id; offering one
   alone frees nothing and leaves the other. Cost if wrong: a user who wants one tag gone keeps it until both go.
4. **The 4 KiB limit of plan 024's reader stays; a manifest over it blocks Ollama's offers**: real manifests are about
   1 KB; raising the limit is a core change for a case not seen. Cost if wrong: no Ollama model is offered on a Mac with
   a model of about twenty layers or more (measured in 04/02).
5. **Any unreadable manifest, any entry not the user's, or a partial download blocks every Ollama offer**, with the
   store's left-out: sharing cannot be told without every manifest, and a pull in progress may be about to name a layer
   that looks unshared. Cost if wrong: an abandoned partial download hides Ollama's offers until it is finished or
   removed.
6. **No `ollama rm`, no HTTP request; the files go to the Trash**: the server must run for its command, a tool never
   runs in a sandbox, and its interface is a network request. Ollama reads its manifests from the disk, so a removed
   manifest is a removed model. Cost if wrong: none for Ollama's consistency (to verify that `ollama list` agrees).
7. **Every Ollama model is Medium** (the deferred item proposed a model made on this Mac "not claimed, or High"):
   nothing in a manifest tells a created model from a pulled one, and the default host and namespace are the same for
   both; rule 1 asks for a higher level when in doubt, as plan 023 chose for LM Studio's imports. Cost if wrong: a
   downloaded model one level too high; a created model at Medium rather than High, with its texts saying it comes back
   only from its files.
8. **Ollama's app or server running keeps nothing**: Ollama's app usually runs all day in the menu bar, so keeping its
   models while it runs would mean never offering them; the cleaner sees a loaded model's mapped layer, and a pull's
   partial file blocks the store. Cost if wrong: a model the server opens in the seconds between the check and the act
   moves while loaded.
9. **Layers no manifest names are not claimed**: in a pull's last seconds every layer exists before its manifest, and
   Ollama removes such layers itself. Cost if wrong: gigabytes left by an abandoned pull stay until Ollama's next start
   (shown in the storage map).
10. **Three interests on the manifests' folders only to read their owner**: the walk lists a folder of another user only
    when it may, and a hidden manifest would make a shared layer look unshared. Cost if wrong: three cheap interests
    more than needed.
11. **The model's date is the newest of its manifests' change and its own layers' access**: Ollama records no last use;
    an access time is lazy and only makes the date newer; nothing is judged by age. Cost if wrong: a model used today
    offered beside an old one, with its date in the reason.
12. **Ollama is a row of plan 023's module with a shape of its own, not a module**: the same section, switch and
    group; the table keeps the place as data (rule 5). Cost if wrong: one module to split later.
13. **Only the default place; `OLLAMA_MODELS` is not read**: plan 018's rule for every cache. Cost if wrong: a moved
    store is not offered (its space is in the storage map).
14. **The confirmation lists the other paths on demand, not by default**: the user confirms one model; its files are
    one click away (rule 7). Cost if wrong: one more line shown by default.
15. **The journal's version goes to 2 for the parts of an item**: a crash in the middle of a model must leave every
    path it was about to move on record. Cost if wrong: one migration.
16. **No new dependency**: `serde_json` (already in the workspace) becomes a normal dependency of the module. Cost if
    wrong: none.

## Conclusions

Built, **awaiting the user's walk**; the numbers, the changes from this design and the calls taken while building are in
the [global readme](./readme.md#conclusions). Of the calls above, 2 (the cancel is read before every part; the permanent
way's partial result has its own text) and 15 (the journal also records `went` and `back`) were refined while building,
and 5 (a link, a hidden folder or any entry that is not a plain-named folder in the manifests' tree blocks the store)
was made stricter; the others stand as written. Open questions: none; the real-Mac facts "to verify" are pending the
user's dry run.
