# Local AI models

**Since:** 2026-11-15 · **Proposed by:** the agent

Taken up by plan [023](../plans/023-local-ai-models/readme.md) (2026-11-21): the models of Hugging Face (models and
datasets, Low) and LM Studio (Medium), one finding per model, moved to the Trash. Ollama's models are built by plan
[029](../plans/029-ollama-models/readme.md) (2026-11-21): one finding per model, its manifests and the layers no other
model uses moved to the Trash together as one item, its size the bytes it frees, every model Medium. Other apps'
models, the Hugging Face chunk cache and the tools' own commands stay ideas. The open questions are answered there:
the folder goes to the Trash, no tool command; datasets are a category of their own.

## Problem

Running language and image models locally has become a normal part of a developer's Mac. Each model is several
gigabytes, people try many of them and forget them, and every tool keeps its own store in its own hidden folder. Tens
of gigabytes disappear without any app pointing at them.

## Direction

- One section, **Local AI models**, one finding per model: its name and tag, the tool that keeps it, the size it frees,
  the last time it was used when the tool records it.
- The known stores (each path also read from the tool's own environment variable or config when it overrides it):
  - Ollama — `~/.ollama/models` (`manifests/` names the models, `blobs/` holds their layers);
  - Hugging Face — `~/.cache/huggingface/hub` (`models--<org>--<name>`, `datasets--…`; `HF_HOME` moves it);
  - LM Studio — its models folder (`~/.lmstudio/models`, older versions `~/.cache/lm-studio/models`);
  - others later through [rules as data](./architecture/rules-as-data.md).
- **Freed bytes, not listed bytes.** Several models share layers (Ollama blobs, Hugging Face blobs behind snapshot
  symlinks). A model frees only the blobs no other model references; the finding says so: "14 GB, of which 9 GB is
  freed — the rest is shared with `llama3:70b`" ([best practices](./best-practices.md), 15). Answered for Ollama by
  plan 029: the finding's size is what removing it frees, and its reason names the model's whole size and the model
  the rest is shared with.
- Removal through the tool's own command where it has one (`ollama rm`, the Hugging Face cache command), so its index
  stays consistent; otherwise the model folder moves to the Trash.
- Danger level **Low**: downloaded again on demand, at the cost of bandwidth — the finding names the size of that
  download ([best practices](./best-practices.md), 1: "how it comes back").

## Risks — when a rule must NOT claim a model

- **A model built locally** (an Ollama model created from the user's own weights file, a fine-tune saved into the
  cache) cannot be downloaded again. If the manifest does not point to a registry, the model is not claimed, or is
  claimed as **High** with "this model was made on this Mac".
- **A gated or private model** comes back only with a login and an accepted licence: still **Low**, but "how it comes
  back" says "needs your Hugging Face login".
- **A model in use**: the tool's server is running and has it loaded — skipped and reported
  ([`../restrictions/deletion-safety.md`](../restrictions/deletion-safety.md), items in use).
- Model folders the user keeps in their own projects (weights next to code, an image-generation app's `models/` inside
  a workspace) are user files: shown in the [large files](../plans/035-large-files/readme.md) view, never claimed by this rule.

Open questions:

- Is the tool's own command required (it may need the tool's server running), or is moving the folder to the Trash
  enough when the tool is not running?
- Datasets in the Hugging Face cache: the same section, or one of their own?
