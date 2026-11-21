# 023 — Local AI models: design

**Status:** ✅ approved (the user's delegation of 2026-11-20) · **Version:** 0.1.0 · **Date:** 2026-11-21 ·
**Modules:** modules, testing, sandbox, e2e, app, app/ui, docs

Written on 2026-11-21 from the idea [Local AI models](../../ideas/local-ai-models.md) and from a read of the code on
plan 028's head (40c669ac), read-only. Approved by the user's delegation of 2026-11-20: after plans 027 and 028, build
plan 023 autonomously, take the recommended decision on every question and record each as an agent's call; the user
checks everything together later. Where the code or the rules forced a different choice than the idea, the design says
so and the choice is an agent's call. Facts marked "to verify" are checked by hand in 03/02.

## Intent

Running language and image models locally is a normal part of a developer's Mac. Each model is several gigabytes,
people try many and forget them, and every tool keeps its own store in a hidden folder. The idea asks for one section,
**Local AI models**, one finding per model, with its name, the tool that keeps it, the size it frees and when it was
last used; danger Low, since a model comes back by download at the cost of bandwidth; and never a model that cannot
come back.

## Goals

1. A module `local-ai-models` (`modules/local-ai-models`, crate `fresher-module-local-ai-models`, group Development)
   that offers, one finding per model, the models in the default stores of **Hugging Face** (its hub cache) and
   **LM Studio** (its models folder, the current place and the older one).
2. Each finding names the model (`owner/name`), the tool, when its files were last opened or changed, and how it comes
   back; its size is what removing it frees.
3. Nothing outside a tool's own store is claimed: a model file in the user's project, a folder that does not have the
   tool's layout, a link, a file loose in the store.
4. Removal moves the model's own folder to the Trash, through the cleaner's existing checks; no new removal kind and no
   tool command.

## Non-goals

- **Ollama's models in this plan** (agent's call 1): its models share layer files across one `blobs/` folder, which
  models use them is written only inside its manifest files, and a model is several files in two folders. The module
  cannot read a file's contents (only the walk reads the head of a `CACHEDIR.TAG`), and a finding has one removal of
  one path. Its space stays visible in the storage map as its real folders. Postponed to
  [`../../deferred/ollama-models.md`](../029-ollama-models/readme.md) with what it needs.
- A store moved by an environment variable or the tool's settings (`HF_HOME`, `HF_HUB_CACHE`, LM Studio's chosen
  folder): only the default places, as plan 018 decided for every cache (agent's call 3). A moved store is missed,
  never mistaken.
- The tools' own commands (`hf cache delete`, `lms`): the folder goes to the Trash (agent's call 2).
- Hugging Face Spaces in the cache, its locks, its newer chunk cache (`~/.cache/huggingface/xet`): not claimed; the
  chunk cache stays an idea.
- Models kept by other apps (image generators, editors with a built-in model): later, through rules as data, as the
  idea says.
- An age threshold or a setting: every model in a store is offered, whatever its age (agent's call 8).

## What already exists

| Need                                    | What exists                                                                                                    | Plan     |
| --------------------------------------- | -------------------------------------------------------------------------------------------------------------- | -------- |
| A place under the home                  | `Pattern::Under(KnownFolder::Home, ".cache/huggingface/hub/*")`, `*` one name; resolved through `KnownFolders` | 002, 016 |
| A folder summed with everything inside  | `Claim::Whole`: `Match.sizes` covers the folder, `Match.inside` (`newest_change`, `newest_file_access`, `special`, `foreign`, `lock_named`, `complete`) | 004, 017 |
| A tool's sign in its folder             | `Interest::holding(names)`, `Match.holds`                                                                      | 018      |
| Links                                   | listed as themselves, never followed; a link inside a whole folder counts as the link, its target is not summed | 004      |
| A data table of places                  | `modules/dev-caches/tools.toml`, parsed and checked when the module is built                                   | 018      |
| An app open keeps a folder              | `cx.system().running_apps()`, a row's bundle id (`dev-caches` keeps derived data while Xcode runs)             | 009, 018 |
| Left-outs                               | `out.left_out(LeftOutItem { category, reason, sizes })`, `with_problems(keys)`                                 | 011      |
| Removal                                 | `Removal::MoveToTrash(path)`; the cleaner's guard (open files inside a folder skip it, check 10)               | 008, 009 |
| A sandbox world with open apps          | `[[case.running]]` in a case file; `[[case.symlink]]`                                                          | 016, 021 |

Nothing in the core is missing for this plan: the module is a data table, a pure decision and texts, like
`dev-caches`.

## Design

### 1. The stores (`stores.toml`)

One row per store, in the order of the module's interests. Every path is relative to the home (`KnownFolder::Home`),
plain names, never an OS path in code (rule 5).

| Row             | Tool         | Place (under `~`)                | Shape             | Sign held by the model folder | App                            | Categories by the name's kind                                              |
| --------------- | ------------ | -------------------------------- | ----------------- | ----------------------------- | ------------------------------ | -------------------------------------------------------------------------- |
| `hugging-face`  | Hugging Face | `.cache/huggingface/hub/*`       | `hub-repo`        | `blobs`, `snapshots`          | —                              | `models` → `hugging-face-models`, `datasets` → `hugging-face-datasets`      |
| `lm-studio`     | LM Studio    | `.lmstudio/models/*/*`           | `publisher-model` | —                             | `ai.elementlabs.lmstudio` (to verify) | `lm-studio`                                                          |
| `lm-studio-old` | LM Studio    | `.cache/lm-studio/models/*/*`    | `publisher-model` | —                             | `ai.elementlabs.lmstudio` (to verify) | `lm-studio`                                                          |

- **Hugging Face's hub cache** keeps one folder per repository: `models--<owner>--<name>` (or `models--<name>` for the
  few repositories without an owner), `datasets--…`, `spaces--…`. Inside: `blobs/` (the files, named by their hash),
  `snapshots/<revision>/` (the repository's files as links into `../../blobs/`) and usually `refs/` (a branch name to
  a revision). Every blob belongs to that one repository; revisions of it share blobs, never two repositories. So the
  folder is the model, and moving it frees everything it sums (agent's call 4). The hub's own `.locks/` and
  `version.txt` sit beside the repositories.
- **LM Studio** keeps `models/<publisher>/<model>/` with the model's files (`.gguf`, or a folder of weights for its
  other format). The older versions used `~/.cache/lm-studio/models` with the same layout (to verify on a Mac with an
  old install). A model the user imported is put in the same layout, under a publisher name the user chose: the module
  cannot tell it from a download (agent's call 6).
- The table is read with `deny_unknown_fields` and refused, one error each, for: no row; an id twice or not kebab
  case; a category that is not the module's or a category of the module no row uses; a place that is not a plain
  relative path or does not end in `*`; a shape the path does not fit (`hub-repo` needs one trailing `*`,
  `publisher-model` two); a `hub-repo` row without its kinds, or a kind twice; a sign name that is not one name; a
  bundle id that is not one.

### 2. Interests

One `Pattern::Under(Home, <place>)` with `Claim::Whole` per row, `.holding(sign)` when the row has a sign: three
interests. A whole claim of `hub/*` takes every entry of the hub, the locks and the Spaces included: they are summed
and never claimed (agent's call 9). Nothing outside the three places is matched; the stores are tools' folders, not
the user's, so no folder of the user's becomes opaque to the storage map.

### 3. Names

- `hub-repo`: the folder's name split on `--`: `<kind>--<owner>--<name>` or `<kind>--<name>`, each part non-empty
  (Hugging Face refuses `--` inside an owner or a name, so the split is exact). Any other shape (`models--`,
  `models----x`, `models--a--b--c`, `.locks`, `version.txt`), or a kind the row does not list (`spaces`), is not a
  model: not claimed. The title is `owner/name`, or `name`.
- `publisher-model`: the publisher and the model folder's names, neither starting with `.`; the title is
  `publisher/model`. A file at the model's depth (a loose `.gguf` in a publisher's folder) is not claimed.
- Names are compared as the OS gives them; a name that is not valid UTF-8 is not claimed (a title must be text).

### 4. The decision (`decide.rs`, a pure function)

For one match of one row, the first row that applies:

| #   | When                                                                                               | Verdict                    |
| --- | -------------------------------------------------------------------------------------------------- | -------------------------- |
| 1   | not a folder (a file, a link, a socket)                                                            | not claimed                |
| 2   | the name does not fit the row's shape, or its kind has no category                                 | not claimed                |
| 3   | the row has a sign and a name of it is not held                                                    | not claimed                |
| 4   | not read in full (`inside` missing or not `complete`), or no time at all                           | not claimed                |
| 5   | a socket, a pipe or a device inside                                                                | left out `special`         |
| 6   | not the user's, a system flag, restricted, the user's lock, or such an entry inside                | left out `not-yours`       |
| 7   | the row names an app and the running apps cannot be listed                                         | left out `no-system`       |
| 8   | the row's app runs                                                                                 | left out `app-open`        |
| 9   | otherwise                                                                                          | offered                    |

Rows 4 to 8 mirror `dev-caches`' rows 4 to 8 (one shape for the same questions). The verdict does not depend on time
or on the order of matches; a restored scan decides the same (nothing is judged by age).

### 5. Findings

- Identity `<row id>/<the shape's folder names>`: `hugging-face/<repository folder>`, LM Studio
  `<row id>/<publisher>/<model>`, since two publishers can ship a model under the same folder name; subject and removal `Subject::Path` and `Removal::MoveToTrash` of the model folder;
  sizes the match's (`logical`, `allocated`, `reclaimable`, with their completeness): what the Trash takes and,
  once emptied, frees.
- Title `local-ai-models.title` with `name` (the title of section 3, `TextArg::Text`).
- Reason `local-ai-models.reason.<row's tool key>` (`hugging-face`, `lm-studio`) with `date`: the newer of
  `inside.newest_change` and `inside.newest_file_access` (`TextArg::Date`), and `size`: the match's logical size
  (`TextArg::Size`), the size of the download that brings it back.
- Left-outs: `LeftOutItem { category, reason: local-ai-models.left-out.<reason>, sizes }` for rows 5 to 8.
- Categories (danger, texts in section 6):

| Category                                | Danger | Why                                                                                       |
| --------------------------------------- | ------ | ----------------------------------------------------------------------------------------- |
| `local-ai-models.hugging-face-models`   | Low    | only Hugging Face's downloader writes this layout; a model comes back by download          |
| `local-ai-models.hugging-face-datasets` | Low    | the same, for datasets                                                                     |
| `local-ai-models.lm-studio`             | Medium | an imported model shares the layout and does not come back by download (agent's call 6)    |

### 6. Texts (`locales/en.json`)

Plain English (rule 7); the drafts, polished when built:

- `local-ai-models.hugging-face-models.name` "Hugging Face models"; `.what` "Language, image and speech models that
  programs downloaded from Hugging Face, one folder per model."; `.made-by` "Programs that use Hugging Face's libraries
  (Python scripts, notebooks, local AI apps)."; `.if-removed` "The model's folder goes to the Trash. A program that
  needs the model downloads it again; until then, folders that link to its files see them missing."; `.comes-back`
  "Yes, by download the next time a program asks for it, which needs the network and can take long for a big model. A
  gated or private model also needs your Hugging Face login."
- `local-ai-models.hugging-face-datasets.*`: the same for datasets ("Datasets that programs downloaded from Hugging
  Face…").
- `local-ai-models.lm-studio.name` "LM Studio models"; `.what` "The models LM Studio keeps to run them on this Mac, one
  folder per model."; `.made-by` "LM Studio, an app that runs language models locally."; `.if-removed` "The model's
  folder goes to the Trash and LM Studio no longer lists it. Your chats and LM Studio's settings stay."; `.comes-back`
  "If you downloaded it in LM Studio, yes: download it there again. A model you added yourself comes back only from
  your own copy."
- `local-ai-models.title` "{name}".
- `local-ai-models.reason.hugging-face` "Downloaded from Hugging Face; last opened or changed on {date}. A program that
  needs it downloads {size} again."
- `local-ai-models.reason.lm-studio` "Kept by LM Studio; last opened or changed on {date}. Downloading it again in LM
  Studio takes {size}; a model you added yourself does not come back."
- Left-outs: `local-ai-models.left-out.not-yours`, `.special`, `.app-open` ("LM Studio is open and may be using its
  models. Quit it and scan again."), `.no-system` — the `dev-caches` sentences adapted.
- The switch: `local-ai-models.enabled.label` "Local AI models", `.description` "Finds models downloaded by Hugging
  Face and LM Studio. Each one goes to the Trash and comes back by download."

### 7. Tests

Negative cases first everywhere (rule 8, [`../../testing.md`](../../testing.md)).

- **Unit**: `stores.rs` (every refusal, then the shipped table pinned), `name.rs` (every shape it refuses, then the
  titles), `decide.rs` (one test per row, keep before offer; properties: the verdict never depends on the order of the
  matches, a restored scan decides the same, a row with no app never asks the running apps), `tests.rs` (the module
  on hand-built matches).
- **Fixtures** (`fixtures/homes/local-ai-models-*.toml`, `run_module`, golden `fixtures/golden/local-ai-models.json`):
  `-not-claimed` (Spaces, locks, `version.txt`, names off the shape, a repo folder without `blobs` or `snapshots`, a
  loose file in LM Studio's publisher folder, a hidden publisher, a link named like a repo, Ollama's store, a model
  file in a project), `-kept` (a blob not the user's, a socket inside, LM Studio open), `-offered` (a model with and
  without an owner, a dataset, LM Studio's two places, a model whose snapshot links point into its blobs).
- **Contract**: `assert_module_contract` with `Catalogue::Required`: three category files with the same dangers,
  every text and every declared left-out used by a fixture run.
- **Sandbox**: `fixtures/sandboxes/local-ai-models.toml` and `local-ai-models-lm-studio-open.toml` (the open app is
  shared by every case of a file), with their goldens; `EXPECTED.md` walked by hand.
- **Core e2e**: `crates/e2e/tests/local_ai_models.rs`: the scan offers each model with its sizes; a cleanup moves a
  Hugging Face model to the sandbox's Trash, its snapshot links with it, and a project's link into the cache stays (as
  a link); a model with an open file is skipped `InUse`; a restored scan offers the same.
- **UI e2e**: Scan's cards for the module's categories and a category's Details (there are no module pages since plan
  027), a finding's title and reason, the review naming Medium when an LM Studio model is selected, a cleanup and put back.

## Risks

- **A link from the user's folders into the cache** (older Hugging Face downloads into a project folder made links to
  the cache's blobs): the walk does not see links pointing in. Removing the model breaks them until a program downloads
  it again. The `if-removed` text says so; the Trash keeps it recoverable (agent's call 12).
- **A model in use**: a program with a model's file open makes the cleaner skip the folder (check 10); LM Studio open
  keeps its models (row 8). A script that loads a model only at its start is not seen between runs: the model comes
  back by download (Low).
- **An imported LM Studio model**: Medium, and both texts say a model you added yourself does not come back.
- **A download in progress**: its file is open, so check 10 skips the folder; an abandoned partial download goes with
  its folder.
- **The bundle id of LM Studio and its older place** are "to verify" in 03/02; a wrong id means LM Studio's models are
  offered while it is open (the cleaner still skips any file it has open).

## Open questions

None.

## Decisions

- The user's delegation of 2026-11-20 (translated): "build plan 027, then the ignore of one item, then 023, then 024,
  autonomously; take the recommended decisions; I check everything together later." This design is approved by it.
- No version bump: 0.1.0 ([`../README.md`](../README.md), "Versions").

## Agent's calls (to check)

Made while writing this design from the code and the rules; each with what it costs if wrong.

1. **Ollama is not in this plan; it is deferred** ([`ollama-models`](../029-ollama-models/readme.md)), against the
   idea, which lists it first. One finding per Ollama model needs (a) which blobs a model uses, written only inside
   its manifest, while a module never reads a file and the walk reads only a `CACHEDIR.TAG`'s head; and (b) a removal
   of a manifest and several blobs as one item, while a finding has one `Removal` of one path. The tool route does not
   fit either: `ollama rm` and `ollama list` need its server running, a tool never runs in a sandbox (so no case file
   could prove it), `ollama list` prints rounded listed sizes, not freed ones, and its HTTP interface is a network
   request (privacy). The whole `~/.ollama/models` as one finding would offer locally made models with the rest. Cost
   if wrong: Ollama's space is shown in the storage map but not offered until the deferred item is built.
2. **Removal moves the model's folder to the Trash; no tool command** (the idea preferred the tool's command). Hugging
   Face's cache has no index beside its folders: a missing repository folder is simply not cached, and its command is
   interactive. LM Studio has no command for it and lists its folder as it finds it. Cost if wrong: none for the
   tools' consistency; a later plan may add the commands.
3. **Only the default places; no environment variable and no config is read** (the idea read `HF_HOME` and the tools'
   overrides): plan 018's rule for every cache; an app started from the Dock does not get the shell's variables
   anyway. Cost if wrong: a moved store is not offered (its space is still in the storage map).
4. **Freed equals listed: one finding per repository folder, not per revision** (the idea's "freed vs listed" sentence
   is not needed): Hugging Face's blobs belong to one repository, and its snapshots link inside the same folder; the
   walk's `reclaimable` already leaves out a clone or a hard link shared elsewhere. The sentence "of which N is
   freed" returns with Ollama. Cost if wrong: a size too high for a model whose blob is hard-linked elsewhere (the
   reclaimable size covers it).
5. **Datasets are a category of their own, Low; Spaces are never claimed** (the idea's open question on datasets):
   the same layout and the same way back, but a different "what". Cost if wrong: one category to merge.
6. **LM Studio's models are Medium** (the idea's Low): the module cannot tell an imported model from a downloaded one
   (rule 1: when in doubt, a higher level), and both texts say so. Hugging Face's stay Low: only its downloader writes
   the hub layout. Cost if wrong: one level too high on a downloaded LM Studio model.
7. **LM Studio's models are kept while LM Studio is open** (`app-open`); Hugging Face has no app, so a program with a
   model's file open is the cleaner's check 10. The bundle id is "to verify". Cost if wrong: a wrong id offers models
   while LM Studio runs (files it holds open are still skipped).
8. **No age threshold, no setting; the date shown is the newest of the folder's changes and its files' accesses**
   ("last opened or changed"): neither tool records when a model was last used (to verify for LM Studio); an access
   time is lazy, so it only ever makes the date newer. A restored scan offers the same (nothing is judged by age).
   Cost if wrong: a model used today is offered beside an old one, with its date in the reason.
9. **`hub/*` is claimed whole, routed by name in the module**: no interest shape matches a name prefix, and adding one
   (`only_prefixes`) is new core surface for three names. The hub's `.locks/`, `version.txt` and Spaces are then summed
   whole and never claimed; they are the tool's own entries, not the user's. Cost if wrong: a Space's folder cannot be
   browsed inside in the storage map.
10. **Signs: a Hugging Face repository folder must hold `blobs` and `snapshots`** (`refs` is absent when a revision
    was asked by its hash); **LM Studio's place is its own sign** (a folder two levels down, no hidden name; there is
    no file every model folder holds). Cost if wrong: a non-model folder in LM Studio's models folder is offered at
    Medium.
11. **One module, group Development, id `local-ai-models`**, three categories by store and kind, as `dev-caches` has
    one per tool. Cost if wrong: a group to change.
12. **Links into the cache from the user's folders are not looked for**: the walk never follows links and keeps no
    target; the `if-removed` text names the effect, and the Trash keeps it recoverable. Cost if wrong: a project that
    linked to a model sees broken links until the model is downloaded again.
13. **A folder read in part, or with no time, is not claimed** (`dev-caches`' row 4): the walk and the access status
    report what was not read. Cost if wrong: fewer offers on a Mac without Full Disk Access (the stores are in the
    home, which needs none).
14. **No new dependency**: `serde`, `thiserror`, `toml` as in `dev-caches`; `proptest`, `serde_json` as
    dev-dependencies. Cost if wrong: none.
15. **No new UI component**: its cards, its findings and its left-outs are the existing ones with the core's
    texts. Cost if wrong: none.
