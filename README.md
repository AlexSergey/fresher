<p align="center">
  <img src="assets/logo.png" width="160" height="160" alt="Mac Fresher logo: a dark leaf with a white spark">
</p>

<h1 align="center">Mac Fresher</h1>

<p align="center">
  <b>Every byte explained. Nothing deleted without you.</b><br>
  A fast, private, open-source disk cleaner for macOS — built in Rust.
</p>

<p align="center">
  <a href="https://github.com/AlexSergey/mac-fresher/stargazers"><img alt="GitHub stars" src="https://img.shields.io/github/stars/AlexSergey/mac-fresher?style=flat-square&logo=github&color=7fe0c2"></a>
  <img alt="macOS 13+" src="https://img.shields.io/badge/macOS-13%2B-1c1d21?style=flat-square&logo=apple">
  <img alt="Rust" src="https://img.shields.io/badge/core-Rust-b7410e?style=flat-square&logo=rust">
  <img alt="Tauri 2" src="https://img.shields.io/badge/Tauri-2-24c8db?style=flat-square&logo=tauri&logoColor=white">
  <img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-3a3a3a?style=flat-square">
  <img alt="Status: early development" src="https://img.shields.io/badge/status-early%20development-d69e2e?style=flat-square">
  <img alt="No telemetry" src="https://img.shields.io/badge/telemetry-none-2ea44f?style=flat-square">
</p>

<p align="center">
  <a href="#why">Why</a> ·
  <a href="#what-it-does">What it does</a> ·
  <a href="#safety-first">Safety</a> ·
  <a href="#privacy">Privacy</a> ·
  <a href="#build-from-source">Build</a> ·
  <a href="#contributing">Contributing</a>
</p>

---

## Why

Your Mac is full, and "System Data" or "Other" tells you nothing. An IDE updated itself and quietly kept gigabytes of
settings from every old version. Docker grew into a black hole. A dozen forgotten `node_modules` sit in projects you
have not opened in a year.

**Mac Fresher shows where your space really went — in plain English — and lets you clean it safely.**

## What it does

- 🔍 **No "Other". Ever.** Every byte belongs to a named category that says what it is, which app made it, and what
  happens if you remove it.
- 🚦 **A danger level on everything** — know the risk before you click.

  | Level      | Meaning                                                    |
  | ---------- | ---------------------------------------------------------- |
  | **Safe**   | Rebuilt automatically. Costs nothing but a little time.    |
  | **Low**    | Rebuilt on demand, at the cost of a download or a rebuild. |
  | **Medium** | Can be restored, but you may need to set things up again.  |
  | **High**   | May hold the only copy of your data.                       |

- 🧠 **Finds what others miss** — stale data of old app versions (JetBrains IDEs and more), developer caches, forgotten
  `node_modules` and `target/` folders, leftovers of apps you already uninstalled, Docker images and volumes.
- 📈 **Watch mode** — every scan is remembered locally, so the next one tells you _which app keeps eating your disk_,
  by how much, and since when.
- ⚡ **Fast** — a parallel Rust scanner that streams results while it works.
- 🖤 **Calm, premium interface** — a graphite, low-noise design that answers one question per screen.

> [!NOTE]
> Mac Fresher is in early development: the repository, quality gates and architecture are in place, and the scanner is
> next. ⭐ **Star the repo** to follow along — it really helps.

## Safety first

The app deletes files from your disk, so safety is designed in, not bolted on:

- **Nothing is deleted without your explicit confirmation** of the exact items, their size and their danger level.
- **Trash first.** Permanent deletion is a separate, deliberate choice.
- **Scanning is read-only** — enforced by the linter: only one module of the codebase is allowed to delete anything.
- **Unknown is not junk.** If Mac Fresher cannot tell what something is, it shows it — it never offers it for removal.
- System folders and other users' files are never touched; Docker data is removed only through Docker itself.

## Privacy

**Nothing leaves your Mac.** No telemetry, no analytics, no crash reports, no update pings — no network requests at all.
This is enforced in the build: no HTTP client is allowed in the dependency tree, the app has no network permission, and
the UI cannot load anything from the internet. It is open source, so you can verify every word of this.

**Nothing happens behind your back.** No background checks, no menu-bar helper, no login item, no reminders. Mac
Fresher runs only while you have it open, scans only when you ask, and removes only what you confirm.

## Build from source

Requirements: macOS 13+, [Rust](https://rustup.rs) (the pinned toolchain installs itself), Node.js 24.

```bash
git clone https://github.com/AlexSergey/mac-fresher.git
cd mac-fresher
npm install        # also installs the git hooks
npm run dev        # run the app
npm run build      # build the .app bundle
```

More commands — tests, coverage, linting — are in [`docs/quality.md`](docs/quality.md) and
[`docs/testing.md`](docs/testing.md).

## How it is built

| Part      | Stack                                                                    |
| --------- | ------------------------------------------------------------------------ |
| Core      | Rust: scanner, rules, SQLite snapshots, cleaner                          |
| Shell     | Tauri 2                                                                  |
| Interface | React + TypeScript                                                       |
| Quality   | Clippy pedantic, cargo-deny, ESLint strict, 90 % coverage, git hooks, CI |

The goals and rules behind every decision: [`docs/goal.md`](docs/goal.md) · [`docs/rules.md`](docs/rules.md) ·
[`docs/architecture/`](docs/architecture/README.md).

## Contributing

Ideas, bug reports and pull requests are welcome. Start with [`docs/goal.md`](docs/goal.md) and
[`docs/rules.md`](docs/rules.md); commits follow [Conventional Commits](https://www.conventionalcommits.org/), and the
git hooks check everything before it lands.

Have an idea for a category Mac Fresher should recognize? Open an issue — the more real-world cases, the better.

## License

Mac Fresher is free and open source, dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your
option. Notices: [NOTICE](NOTICE).

Created by [Serhii Aleksandrov](https://github.com/AlexSergey) and the Mac Fresher contributors.
