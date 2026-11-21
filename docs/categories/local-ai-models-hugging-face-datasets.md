# Hugging Face datasets

**Danger level:** Low · **Rule:** none (the module's code, `modules/local-ai-models`) · **Since:** 0.1.0

## What it is

Datasets that programs downloaded from Hugging Face to train or test models, one folder per dataset. Made by programs that use Hugging Face's libraries (Python scripts, notebooks, training tools).

## Where it lives

`~/.cache/huggingface/hub/datasets--<owner>--<name>` (or `datasets--<name>`): one finding per dataset, titled `owner/name`. Only the tool's default place is looked at: the app reads no environment variable and no config file. Models (`models--…`) are their own category.

The tool's sign: the folder holds both `blobs` and `snapshots`. Without them the folder is not claimed.

## What happens if you remove it

The dataset's folder goes to the Trash. A program that needs the dataset downloads it again; until then, folders that link to its files see them missing. Yes, by download the next time a program asks for it, which needs the network and can take long for a big dataset. A gated or private dataset also needs your Hugging Face login.

## How the app decides

The module `local-ai-models` (code with a data table, `modules/local-ai-models/stores.toml`). For each model folder, the
first answer that applies:

| #   | Situation                                                                              | Result                                          |
| --- | -------------------------------------------------------------------------------------- | ----------------------------------------------- |
| 1   | the found item is not a folder (a file, a link, a socket)                              | not claimed                                     |
| 2   | its name does not fit the layout of the store: `datasets--` plus an owner and a name; any other shape is not claimed | not claimed                                     |
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
