# 038/03 — The recorder

**Status:** 🔨 in progress, awaiting the user's walk · **Version:** 0.1.0 · **Date:** 2026-11-22 · **Modules:** scripts, repo, docs

`npm run demo:check` and `npm run demo:record`: one script that generates the world, starts the development build in
demo mode, and either only reads the outcome or also captures the screen and encodes the video and the GIF within the
budget; and the repository's rules for the GIF. The design: [`../00-design.md`](../00-design.md), section 3, agent's
calls 4, 6, 7 and 15.

## Steps

| #   | Step                                         | Status      | Shared files                                                                                               |
| --- | -------------------------------------------- | ----------- | ---------------------------------------------------------------------------------------------------------- |
| 01  | [The recorder script](./01-recorder.md)      | 🔨 in progress | `package.json`, `scripts/check-staged.ts`, `.gitattributes`, `.gitignore`, `docs/hacks/README.md`, `docs/dependencies.md` |

## Conclusions

`scripts/demo-record.ts` is built and unit tested with a fake spawner (113 script tests); its glue (`main`, `play`) is
untested by design because it opens the app and the screen, and it was never run: the first run is the user's walk
(manual-testing 1.21, 2.19). The 5 MiB limit for `assets/demo/*.gif` is in `docs/hacks/`, `.gitattributes` marks GIFs
binary, `ffmpeg` is in `dependencies.md`. Review found and fixed two real faults in the glue: the app was left running
(only the `npm` parent was stopped: the process group is signalled now) and a half-written window file was read.
