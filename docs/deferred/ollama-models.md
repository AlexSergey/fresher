# Ollama's models

**Agreed:** 2026-11-21, by the agent under the user's delegation of 2026-11-20 (plan 023, agent's call 1; to check) ·
**Due at:** after plan 024, or the first plan that lets the walk read a small file's contents

## What is postponed

Plan [023](../plans/023-local-ai-models/readme.md) offers the models of Hugging Face and LM Studio, one finding per
model. Ollama's models (`~/.ollama/models`) are not offered: its space is shown in the storage map as its real folders.
One finding per Ollama model needs what the core does not have:

- **Which layers a model uses.** `manifests/<registry>/<namespace>/<name>/<tag>` names a model; which files of
  `blobs/` (named `sha256-<hash>`) it uses is written only inside that manifest (a small JSON file). A module never
  reads a file, and the walk reads only the head of a `CACHEDIR.TAG`.
- **Shared layers.** Models share blobs (a base model and its variants): a model frees only the blobs no other
  manifest names, and the finding says so ("14 GB, of which 9 GB is freed; the rest is shared with `llama3:70b`").
- **One item of several paths.** Removing a model is its manifest and its unshared blobs, in two folders; a finding
  has one `Removal` of one path.

The tool's own command does not fit either: `ollama rm` and `ollama list` need its server running, a tool never runs
in a sandbox (so no case file could prove it), `ollama list` prints rounded sizes of every layer, not the freed ones,
and its HTTP interface is a network request (privacy).

## What doing it needs

- The walk reads the first bytes of a matched file for an interest that asks it (bounded, as `CACHEDIR.TAG`'s
  check), and the scan cache keeps them, so a restored scan answers the same; or a port the module asks in `finish`
  that reads one small file of a store beneath a held folder.
- A removal of several paths as one confirmed item (each path checked by the guard), or one finding per model that
  moves its manifest and one finding per unshared blob grouped under it.
- A model made on this Mac (a manifest that names no registry the user can pull from) is not claimed, or claimed High
  ("this model was made on this Mac"); a model the server has loaded is skipped (the cleaner's open-files check sees
  the mapped blob).
- `fixtures/sandboxes/` cases with manifests and shared blobs; the category doc; the idea's "freed bytes" sentence.
