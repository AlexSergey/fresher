# Ollama models

**Danger level:** Medium · **Rule:** none (the module's code, `modules/local-ai-models`) · **Since:** 0.1.0

## What it is

The language models Ollama keeps to run them on this Mac: for each model a small list of its files and the layer files
it names. Models can share layers. Made by Ollama, an app and a command that run language models locally.

## Where it lives

The store `~/.ollama/models`, which holds `manifests/` and `blobs/`. A model's list (its manifest) is
`manifests/<host>/<namespace>/<model>/<tag>`, a small JSON file naming the model's config and layers by digest and
size; a layer is `blobs/sha256-<64 hex digits>`. One finding per model, titled the way Ollama shows it: `model:tag` for
Ollama's own library, `namespace/model:tag` for another namespace of its registry, `host/namespace/model:tag` for
another host. Tags whose manifests name exactly the same files (`llama3:latest` and `llama3:8b`) are one model, one
finding titled by every tag. Only the default place is looked at: the app reads no environment variable
(`OLLAMA_MODELS`) and no config file.

Once rows 1 to 3 below and the first half of row 4 have let the store through, the module reads every manifest its scan
matched, each once, through the small-file reader (at most 4 KiB, never through a link), to tell which layers each model
names. It reads no other file, runs no `ollama` command and sends no request.

## What happens if you remove it

The model's manifests and the layers no other model names go to the Trash together, as one item: the manifest first,
then the layers, every one checked before the first moves. Ollama no longer lists the model. Layers another model uses
stay, so your other models keep working. If you downloaded it, it comes back: `ollama pull` with its name downloads it
again. A model you made yourself with `ollama create` comes back only from its Modelfile and your files, or from the
Trash. Put back brings the layers back first and the manifest last.

## How the app decides

The module `local-ai-models` (code with a data table, `modules/local-ai-models/stores.toml`). The store first, once a
scan; the first answer that applies decides for every model:

| #   | Situation                                                                                                  | Result                                         |
| --- | ---------------------------------------------------------------------------------------------------------- | ---------------------------------------------- |
| 1   | no `~/.ollama/models`, or it does not hold both `manifests` and `blobs`                                    | not claimed                                    |
| 2   | a folder of the manifests' tree, a manifest, a layer or a partial download is not yours, locked or restricted | left out: removing it needs an administrator   |
| 3   | `blobs/` holds a download not finished (`sha256-<hex>-partial`, or that name and `-<n>`)                  | left out: Ollama is downloading a model        |
| 4   | an entry at a host's, namespace's or model's depth of `manifests/` may hide manifests the app never reads (a link, a file of a plain name, a folder whose name starts with a dot), or a manifest could not be read: not JSON, not schema 2, a digest off its shape, over 4 KiB, a link, gone | left out: which files are shared is unknown    |

Then each model, in name order:

| #   | Situation                                                                                                  | Result                                         |
| --- | ---------------------------------------------------------------------------------------------------------- | ---------------------------------------------- |
| 5   | a layer it names is missing, is not a regular file, or is not of the size its manifest lists               | left out: the model is incomplete              |
| 6   | more than 64 parts (manifests and freed layers), fewer than two (one tag whose every layer is shared), or no time known | not claimed                                    |
| 7   | otherwise                                                                                                  | **offered**                                    |

A store left out is one line with the sizes of its manifests, layers and partial downloads (a manifest under a hidden
folder is not counted); an incomplete model is one line with the sizes of its manifests and the layers it would free. A
file in `blobs/` of another name, anything in `manifests/` off the four-level shape, and a hidden file (`.DS_Store`) at
a host's, namespace's or model's depth are neither counted nor claimed; any other entry there that is not a folder of a
plain name is row 4.

A model's size is what removing it frees: its manifests and the layers no other model names. Its reason says when it
was last opened or changed (the newest of its manifests' last change and its freed layers' last access) and, when other
models share its layers, the model's whole size, the first of them by name and how many more: those layers stay. An
incomplete model's layers still count as named, so they never leave with another model.

Age is not judged: every model is offered. The danger level is Medium because a model made with `ollama create` looks
like a downloaded one and does not come back by download. A model Ollama has loaded maps its layers, so the cleaner
skips the whole model before anything moves; a pull in progress writes a partial file, so nothing is offered then.
Ollama's layout (the default host and namespace, the partial names, where it reads manifests) is to be verified on a
real Mac.
