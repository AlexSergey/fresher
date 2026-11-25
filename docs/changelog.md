# Changelog

What changed in each release of Fresher, for the people who use it, newest first. Versions follow semantic
versioning; until 1.0.0 a minor version may change how the app behaves. How a release is cut:
[`release.md`](./release.md). The notes of each release: [`releases/`](./releases/).

## [Unreleased]

## [0.1.0] - 2026-11-25

The first release, for testers: please try it on your Mac and tell us what it got wrong.

### What Fresher finds

- **One scan of your home folder** shows your disk as a ring: what is free, what Fresher can free, your files and
  apps, macOS, and the space the system keeps for itself. The scan only reads; it changes nothing on your disk.
- **What you can remove is sorted by what removing it costs**: Safe to remove, Usually safe, Look before removing.
  Every kind of file says what it is, which app made it, what happens if you remove it, and whether it comes back.
- **No "Other"**: space Fresher does not recognize is never put in a catch-all bucket.
- **Developer caches**: npm, Yarn, Homebrew, Cargo, CocoaPods, Gradle, pip and Go's build cache, and Xcode's build
  data.
- **Old versions of JetBrains IDEs**: the caches, logs and settings each older version left behind, and the data of an
  IDE you removed.
- **Apps you removed**: the caches and settings they left in your Library.
- **Caches of the apps you use**, including browsers and apps built on a web engine.
- **Downloads**: unfinished and paused downloads, installers of apps you already have, old macOS installers.
- **Old projects**: the packages, build output, Python environments and Rust build caches of projects you have not
  changed in a long time.
- **Local AI models**: Hugging Face models and datasets, LM Studio and Ollama models; a layer two Ollama models share
  is counted once.
- **Unused Rust and Node versions** that no project of yours asks for.
- **Xcode**: device debug files, previews and simulator caches.
- **Old logs, crash reports and temporary files.**
- **Docker**: what images, containers, volumes and the build cache take, with the command that frees each part for you
  to run. Fresher never runs it and never removes anything of Docker's.
- **What it left out, and why**: an item held back for now (its app is open, it changed lately) is shown with the
  reason; an item Fresher keeps for good (the IDE version you use, an installed app) is not offered at all.

### How it keeps you safe

- **Nothing is removed until you confirm** the exact items, with their total size and the highest danger level among
  them.
- **Items go to the Trash.** A permanent delete is a separate choice you make each time.
- **Before your first cleanup** Fresher asks you to accept five plain lines: no warranty, each removal is your
  decision, a danger level is a best judgement, keep a backup, and what the Trash can and cannot undo.
- **Every item is checked again right before it moves**: a link, a protected place, an item that changed or is in use
  since the scan is skipped and named, never removed.
- **The riskiest kinds of files are not offered in this release**: big downloads you never opened, the data of apps
  you removed, Xcode archives and your own large files stay out until the app has been tested more. The Largest screen
  is hidden for the same reason.
- **Ignore anything**: an item, a kind of file, a module or a whole group; ignored items are never offered or removed,
  and Settings brings them back.

### The Trash and putting things back

- **Cleanups** lists every cleanup, newest first, with what is still in the Trash; put back one item or all of them,
  and they return to where they were.
- **The space comes back when the Trash is emptied**: the scan shows what waits in the Trash and how much of it came
  from Fresher's cleanups, and opens the Trash for you. Fresher never empties it.

### Apps

- **The Apps screen** lists your apps, largest first, with what each one keeps on your disk and, when macOS knows it, when you last used it.
- **Uninstall an app** and keep its settings and data: the app goes to the Trash as one item you can put back. Apple's
  apps, open apps and apps that need an administrator or are managed by a package manager are shown with the reason
  and not offered.

### Privacy

- **Nothing leaves your Mac**: no network requests at all, no telemetry, no crash reports, no update checks.
- **Nothing in the background**: no helper, no login item, no scheduled scans. Opening the app starts nothing; it
  scans only when you ask.
- **Full Disk Access is asked for only when a scan needs it**, with the reason. Without it Fresher still works and says
  which folders it could not see.
- **Your data stays in the app's own folder** in `~/Library/Application Support/`. Fresher reads names and sizes, not
  the contents of your files.

### Settings

- **Settings** with its own sidebar and a search: the folders to scan, what Fresher looks for and what it ignores,
  the folders macOS guards (Desktop, Documents, Downloads), motion, and About with the version and the licences of
  everything Fresher is built from.

### Downloads

- **One download for every supported Mac**: a disk image with a universal app for Apple silicon and Intel, macOS 13
  or later.
- **Checksums and a build provenance attestation** for every release, so you can check the download is the one built
  from this repository.
- **Not signed with an Apple Developer ID yet**: macOS asks you to allow the first launch
  ([release notes](./releases/0.1.0.md)).

[Unreleased]: https://github.com/AlexSergey/fresher/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/AlexSergey/fresher/releases/tag/v0.1.0
