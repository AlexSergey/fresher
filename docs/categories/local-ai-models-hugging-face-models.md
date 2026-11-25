# Hugging Face models

**Danger level:** Low · **Rule:** none (the module's code, `modules/local-ai-models`) · **Since:** 0.1.0

**Example:** `~/.cache/huggingface/hub/models--<owner>--<name>`

## What it is

AI models for text, images or speech that programs downloaded from Hugging Face, one folder per model. Made by programs
that use Hugging Face, such as Python scripts and local AI apps.

## Where it lives

`~/.cache/huggingface/hub/models--<owner>--<name>` (or `models--<name>` for a model without an owner): one finding per model, titled `owner/name`. Only the tool's default place is looked at: the app reads no environment variable and no config file. Datasets (`datasets--…`) are their own category.

The tool's sign: the folder holds both `blobs` and `snapshots`, the two parts Hugging Face's downloader writes. Without them the folder is not claimed.

## What happens if you remove it

A program that needs the model downloads it again; until then, links to its files stop working. Yes, by download when a
program asks for it, which can take long; a private model also needs your Hugging Face login.

## How the app decides

The module `local-ai-models` (code with a data table, `modules/local-ai-models/stores.toml`). For each model folder, the
first answer that applies:

| #   | Situation                                                                              | Result                                          |
| --- | -------------------------------------------------------------------------------------- | ----------------------------------------------- |
| 1   | the found item is not a folder (a file, a link, a socket)                              | not claimed                                     |
| 2   | its name does not fit the layout of the store: `models--` plus an owner and a name; any other shape, or a kind the store does not list, is not claimed | not claimed                                     |
| 3   | the tool's sign is missing (above)                                                     | not claimed                                     |
| 4   | the folder could not be read in full, or no time of it is known                        | not claimed                                     |
| 5   | a socket, a pipe or a device inside                                                    | left out: a running program may be using it     |
| 6   | not yours, locked by the system, restricted or locked by you, or something inside is   | left out: removing it needs an administrator    |
| 7   | otherwise                                                                              | **offered**                                     |

A folder that is not claimed is not shown and nothing is said about it. Age is not judged: every model is offered, with
the date it was last opened or changed in its reason. Hugging Face has no app, so no open app keeps a folder; a program
holding a file of it open when you confirm makes the cleaner skip it. The folder's size is what the Trash takes: its
blobs belong to it alone. Never claimed beside the folders offered: the hub's `.locks` and `version.txt`, Spaces
(`spaces--...`), a name off the layout, and a folder without both `blobs` and `snapshots`. The folder goes to the
Trash; the app never runs the tool's own command.
