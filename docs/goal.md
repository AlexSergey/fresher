# Goal

## The problem

Disk space on a Mac disappears and it is not clear where it went or what is safe to remove. Generic cleaners either
show a large "Other" bucket, or delete things without explaining what they are and what it costs to lose them.

A real case that started the project: WebStorm updated itself to 2026.3, and the settings, caches, logs and plugins of
every previous version stayed on disk, taking gigabytes, without any tool pointing at them.

## What the product is

Mac Fresher is a desktop application for macOS that finds where disk space goes, explains it in plain English, and
cleans only what the user explicitly confirms.

- **macOS only.** Other operating systems are out of scope.
- **Fast.** The core is Rust: a parallel file system scan, no blocking of the UI, results streamed as they are found.
- **A Tauri 2 shell** with a React + TypeScript UI over the Rust core.

## What it must achieve

### 1. Every byte is explained — no "Other"

- Every finding belongs to a named category written in plain English: what it is, which app made it, what happens if it
  is removed, and whether it comes back.
- There is no "Other", "Miscellaneous" or "System data" catch-all. Space no rule recognizes is shown as the real folders
  that hold it, never as an anonymous bucket.

### 2. Every category has a danger level

Each category carries a danger level that says what the user risks by removing it (the levels are defined in
[`categories/README.md`](./categories/README.md)). The level, the explanation and the consequence are shown before any
deletion, not hidden behind a details button.

### 3. Nothing is deleted without explicit confirmation

- Scanning never changes the disk.
- Deletion happens only after the user confirms the exact list of items, with their total size and the highest danger
  level among them.
- Items go to the Trash by default; a permanent delete is a separate, explicit choice.

### 4. It understands the hard cases

Beyond plain caches, the rules recognize leftovers that generic tools miss, for example:

- **Stale versions of an app's data**: JetBrains IDEs (WebStorm, IntelliJ IDEA, PyCharm, …) keep settings, caches, logs
  and plugins per version; once a newer version is installed, the older folders are leftovers.
- **Developer caches**: package manager caches (npm, Yarn, pnpm, Cargo, Homebrew, CocoaPods, Gradle, …), Xcode derived
  data, archives and old simulators.
- **`node_modules` folders** of projects, with the project they belong to and when it was last touched.
- **Docker**: images, containers, volumes and build cache. This is a dangerous zone: a volume may hold the only copy of
  a database; each kind gets its own danger level and explanation.
- **App caches and logs** in `~/Library`.

### 5. Watch mode: who keeps growing

The app keeps its own state in small versioned files in its own folder, with nothing to install (plan
[002](./plans/002-base-architecture/readme.md)). It keeps the last two scans (plan 004) and a history point per scan
(plan 010); a scan is compared against them and the app shows which apps and categories grew, by how much, and since
when — so an app that keeps accumulating junk is named, not guessed.

### 6. A simple, premium interface

- Simple and intuitive; not overloaded with data. The first screen answers "where did my space go" in one look.
- Visual direction **Dark Premium**: graphite background, soft surfaces, thin borders, careful typography, expressive
  charts, minimal visual noise.

### 7. Complete privacy and security

The app sees the whole disk, so it must be trustworthy by construction, not by promise (the user, 2026-11-15:
"complete privacy; nothing is used to send data out").

- **Nothing leaves the Mac.** No network requests of any kind: no telemetry, analytics, crash reports, update checks,
  remote configuration, fonts or images from the internet. The app works the same with the network off.
- **Enforced, not just promised**: no HTTP client in the dependency tree (cargo-deny bans them), no network permission
  in the Tauri capabilities, a Content Security Policy with no remote origin in the UI
  ([`restrictions/privacy.md`](./restrictions/privacy.md)).
- **Local data stays local and minimal**: the scan cache, history and settings live only in the app's own folder in
  `~/Library/Application Support/`; file contents are never read beyond what a rule needs to identify a folder, and
  never stored or logged.
- **Least privilege**: Full Disk Access is asked for only when needed, with a plain-English reason; the app works with
  reduced coverage without it and says what it could not see.
- **Open and verifiable**: the project is free and open source (MIT OR Apache-2.0), so anyone can check every claim
  above in the code.
- **Supply chain**: exact dependency versions, locked builds, licence and security advisory checks on every change
  ([`quality.md`](./quality.md)).

### 8. Transparent: it does only what the user asks

The app does nothing behind the user's back (the user, 2026-11-15: "everything must be transparent — only what the user
considers necessary").

- **No background work.** The app runs only while the user has it open: no background checks, no menu-bar helper, no
  launch agent, no login item, no scheduled scans, no reminders or notifications that the app decides on.
- **Every action starts with the user**: a scan when the user starts it, a cleanup when the user confirms it. Quitting
  the app stops everything.
- **Everything it does is visible**: what is being scanned, what was found and why, what will be removed and how, and
  afterwards what was removed — nothing hidden in a log only.

## Non-goals

- Other operating systems.
- Antivirus, "speed boosters", RAM cleaning, startup item managers.
- Automatic or scheduled deletion without the user.
- Background checks, alerts, reminders, a menu-bar or login helper — the app works only while the user has it open.
- Any network access or telemetry: everything stays on the Mac.
