# LM Studio models

**Danger level:** Medium · **Rule:** none (the module's code, `modules/local-ai-models`) · **Since:** 0.1.0

## What it is

The models LM Studio keeps to run them on this Mac, one folder per model. Made by LM Studio, an app that runs language models locally.

## Where it lives

`~/.lmstudio/models/<publisher>/<model>`, and the older place `~/.cache/lm-studio/models/<publisher>/<model>`: one finding per model folder, titled `publisher/model`. Only the tool's default place is looked at: the app reads no environment variable and no config file. A model you imported into LM Studio sits in the same layout under a publisher name you chose.

The tool's sign: the place itself. A folder two levels down in LM Studio's models folder, with no hidden name, is a model; there is no file every model folder holds. A loose file in a publisher's folder is not claimed.

## What happens if you remove it

The model's folder goes to the Trash and LM Studio no longer lists it. Your chats and LM Studio's settings stay. If you downloaded it in LM Studio, it comes back: download it there again. A model you added yourself comes back only from your own copy.

## How the app decides

The module `local-ai-models` (code with a data table, `modules/local-ai-models/stores.toml`). For each model folder, the
first answer that applies:

| #   | Situation                                                                              | Result                                          |
| --- | -------------------------------------------------------------------------------------- | ----------------------------------------------- |
| 1   | the found item is not a folder (a file, a link, a socket)                              | not claimed                                     |
| 2   | its name does not fit the store's layout (a publisher and a model, neither starting with a dot) | not claimed                                     |
| 3   | the tool's sign is missing (above)                                                     | not claimed                                     |
| 4   | the folder could not be read in full, or no time of it is known                        | not claimed                                     |
| 5   | a socket, a pipe or a device inside                                                    | left out: a running program may be using it     |
| 6   | not yours, locked by the system, restricted or locked by you, or something inside is   | left out: removing it needs an administrator    |
| 7   | the open apps could not be listed; LM Studio's store names its app | left out: the app list is unknown               |
| 8   | LM Studio is open (its bundle id is among the running apps) | left out: the app may be using it               |
| 9   | otherwise                                                                              | **offered**                                     |

A folder that is not claimed is not shown and nothing is said about it. Age is not judged: every model is offered, with
the date it was last opened or changed in its reason. The danger level is Medium because an imported model shares the
layout of a downloaded one and the app cannot tell them apart. The bundle id and the older place are to be verified on a
real Mac. The model's folder goes to the Trash; the app
never runs the tool's own command. A model a program holds open when you confirm is skipped.
