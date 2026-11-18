# 018/02/02 — The decision

**Status:** ✅ approved · **Version:** 0.1.0 · **Date:** 2026-11-18 · **Modules:** modules

The design's section 3, row by row, as a pure function; the matches of a scan gathered and decided in `finish`.

## Files

- `modules/dev-caches/src/name.rs` (new) — `project(name: &OsStr) -> Option<&OsStr>` for the `project-hash` shape: the
  name splits at its last `-` into a non-empty project and exactly 28 ASCII lowercase letters; compared on bytes, so a
  name that is not UTF-8 is read too (its project shown with `to_string_lossy`).
- `modules/dev-caches/src/gathered.rs` (new) — the matches of one scan, keyed by its cancel token as
  `modules/jetbrains/src/uninstalled.rs`'s `Gathered` is (a scan cancelled before `finish` leaves nothing to the next):
  the candidates (a `Role::Cache` match: its row, path, kind, sizes, facts, `inside`, `holds`, `marker`) and the sign
  holders (a `Role::SignHolder` match: its index, kind, `holds`, `marker`). `take(cancel)` hands them out once.
- `modules/dev-caches/src/decide.rs` (new) — `decide(candidate: &Candidate, row: &Cache, sign: Option<&Sign>, running:
  Result<&RunningSet, NoSystem>) -> Verdict`, where `Sign` is the `holds` and `marker` of the folder that must hold
  the sign (the candidate's own for a row without `sign-at`, the holder's otherwise, `None` when that holder was not
  matched or is not a folder), and `Verdict` is `NotClaimed`, `LeftOut(Reason)` or `Offered`. Rows 1 to 9 in order:
  1. the candidate is not `EntryKind::Folder`: `NotClaimed`;
  2. the row has a `name` shape and `name::project` refuses the name: `NotClaimed`;
  3. `sign` is `None`, misses a name of `row.sign`, or `row.sign` names `CACHEDIR.TAG` and `marker` is not
     `Some(MarkerCheck::Valid)`: `NotClaimed`;
  4. `inside` is `None` or not `complete`: `NotClaimed`;
  5. `inside.special`: `LeftOut(Special)`;
  6. `!facts.owned_by_user`, `facts.system_immutable`, `facts.restricted`, `facts.user_immutable` or
     `inside.foreign`: `LeftOut(NotYours)`;
  7. the row has an app and `running` is `Err`: `LeftOut(NoSystem)`;
  8. the row's app is among the running apps' bundle ids: `LeftOut(AppOpen)`;
  9. `Offered`.
  `inside.lock_named`, the times and `is_restored` are not read (design, agent's calls 8 and 9).
- `modules/dev-caches/src/lib.rs` — `on_match`: the role of `matched.interest`; a candidate or a sign holder added to
  the scan's `Gathered`; a match of no role ignored. `finish`: for each candidate, sorted by path (matches come in no
  order), `decide`, then `out.left_out(LeftOutItem { category: row's, reason, sizes })` or one `NewFinding` (design,
  section 3: identity `<row id>/<folder name>`, `Subject::Path`, `Removal::MoveToTrash` of the same path, the match's
  sizes, the title `dev-caches.title.<row id>` with `{project}` for the Xcode row, the reason
  `dev-caches.finding.reason-download` or `-build` with `{tool}`, `for_app(app)` when the row has one). The running
  apps are read once per `finish`, only when a row with an app has a candidate.

## Produces

The module's answers, for 02/03's fixtures and every later test.

## Checks

- `name.rs`, negatives first: `Demo-abc`, 27 and 29 letters, an upper-case letter, a digit, `-<28>` (no project),
  `Demo` (no dash) refused; then `Demo-<28>` gives `Demo`, `My-App-<28>` gives `My-App`, a non-UTF-8 project is read.
- `decide.rs`, negatives first, each row's keep before its offer: a file, a link and an `Other` entry; a bad shape; no
  sign holder, a sign holder that is a link, one name missing, a tag `Invalid` and one `None`; an incomplete and a
  missing `inside`; a socket inside; another owner, a system flag, `restricted`, a user lock, a foreign entry inside;
  running apps failing; Xcode open. Then the offers: every row's shape, a sign with extra names. Precedence: not a
  folder over every row, the sign over the inside, special over not-yours, not-yours over the app rows.
- Property (`mod property`): adding names to a sign never turns an offer into a keep; any running set without the
  row's app never keeps a candidate the empty set offers.
- `lib.rs` (unit, with plain `Match` values): the same matches in two orders give the same findings and left-outs; a
  Cargo candidate whose holder arrives after it is offered; a scan cancelled before `finish` leaves nothing to the next.
- `cargo nextest run -p fresher-module-dev-caches`; clippy.

## Conclusions

Built (6544794): `name::project` (the last `-` splits a non-empty project from exactly 28 ASCII lowercase letters),
`Gathered` (candidates and sign holders of one scan, keyed by the cancel token as `jetbrains`' is, handed out once and
sorted), `decide` (rows 1 to 9 of the design on plain values, returning `NotClaimed`, `LeftOut(Reason)` or `Offered`),
and the module's `on_match` and `finish`: a match becomes a candidate or a holder by its interest's role; `finish`
reads the running apps once, and only when a candidate's row names an app, then offers each candidate as a finding
(identity `<row id>/<folder name>`, `Subject::Path` with `MoveToTrash` of the same path, the title with `{project}`
for derived data, the download or build reason with `{tool}`, `for_app` Xcode) or as a left-out item of its category.

Deviations and calls:

- `name::project` returns `&str` and refuses a name that is not UTF-8 (preflight F6), as the design says.
- `decide` takes the row too, and the running apps as `Result<&RunningSet, NoSystem>`, `NoSystem` being the module's own
  one-line type (F15); the running apps are read only for a row with an app (F17), and `finish` passes
  `Err(NoSystem)`, never read, when no candidate names an app, so a future bug keeps a cache instead of offering it.
- `Scan::holder` hands a sign holder out only when exactly one was matched for that sign place; zero or two give none,
  so the answer does not depend on match order and errs safe. Stricter than the brief; the rescan flows of 03/02 show
  Cargo and Homebrew still offered after a rescan.
- `src/home.rs` (test-only) holds the home the two suites share, a file the brief does not list.
- The two `dead_code` expectations of 02/01 are removed; `proptest` is the dev-dependency added here.

Tests (83 in the crate, negatives first, two proptest properties): the name shapes; `Gathered`; each row's keep before
its offer, the precedence (not a folder over everything, the sign over the inside, special over not-yours, not-yours over
the app), a tag `Invalid` or `None`, facts the brief does not list not read; the module's flows (every reason with its
key and sizes, order independence, a holder arriving after its cache, a cancelled scan leaving nothing). RED against
stubs for each file; mutations of the two properties were caught once the strategies were weighted toward offerable
states. `cargo nextest run -p fresher-module-dev-caches`: 84 passed; clippy and fmt clean. Parked minors: an
out-of-range row is skipped silently in `finish`; no reverse safety property (an offer implies every condition); a
candidate matched twice is not de-duplicated (theoretical).
