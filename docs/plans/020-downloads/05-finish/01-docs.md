# 020/05/01 — The docs

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** docs

Every doc the plan's change affects, as built (the docs a step already updated are checked, not rewritten).

## Files

- `docs/architecture/modules.md` — `DaySetting` (01/01), `only_suffixes` (01/02), `receipts()`, `system_version()`
  and `InstalledApp.changed` (01/03), checked; the module `downloads` as the example of a module whose categories need
  a guarded folder (`Need::required(Capability::Folder(_))`) and that reads one folder's first level with an `Item`
  star beside a whole suffixed star; the checklist gains "a module with a days setting uses `DaySetting`".
- `docs/architecture/README.md` — the crate list with `fresher-module-downloads`; the ports with `InstallRecords`; the
  IPC's `Build::Sandbox.simulated`; the groups with **Your files**.
- `docs/architecture/permissions.md` — "How a module declares a need": `downloads` is the first module with a
  required guarded folder; the first scan after an opt-in runs it (01/04); sandbox mode's access from the world
  (02/01), checked.
- `docs/testing.md` — `access`, `macos`, `[[case.receipt]]`, `[[case.app]] changed` (02/01, 02/02), the three case
  files (04/01) and `downloads.rs` (04/02), checked.
- `docs/categories/downloads-*.md` — as built (suffixes, names, texts, the decision); the catalogue's rows; the danger
  table's examples of `docs/categories/README.md` gain "an unfinished download" (Low) and "a big download you have not
  opened" (High) if they fit.
- `docs/ideas/downloads-and-installers.md` — its line on plan 020: what was built; what stays an idea (folders outside
  `~/Downloads`, archives already expanded, reading a package's own contents). `docs/ideas/README.md`: its row.
  `docs/ideas/large-files-finder.md` — a line: the large-untouched rule (the status change as "not opened") exists in
  `downloads`.
- `docs/deferred/` — the shared day setting removed (01/01), the sandbox world updated (02/01, 02/02), checked.
- `docs/hacks/README.md` — "Sandbox mode avoids the places that prompt" (02/01), checked.
- `docs/plans/020-downloads/` — every step's conclusions written as it was built (checked here); the design's tables
  brought up to what was built.

## Checks

- `npm run lint` (Markdown and links); `npx prettier --check` on the changed docs; every relative link of the changed
  files resolves.

## Conclusions

The docs were checked against the code, and most were already written by the steps that built the code (`DaySetting`,
`only_suffixes`, the install records, the sandbox's access and installs, the hacks entry, the deferred items, the five
category files and their rows). Written here: the module `downloads` as the example of a module with a required guarded
folder in `docs/architecture/modules.md` and its checklist line on `DaySetting`; the crate list and the banner's
`Build::Sandbox.simulated` in `docs/architecture/README.md`; `downloads` in "How a module declares a need" of
`permissions.md`; the core e2e flows and the `FakeAccess` that remembers a try in `docs/testing.md`, with the line that
the "not-asked turns granted after the opt-in" proof lives in the core e2e (a case file cannot opt in); the idea file's
"built" and "still an idea" parts, the ideas row and the large-files line; two rows of the danger table's examples; the
removed deferred item's links (019's step 02/01 and this plan's own files). Left for the user, by hand: the scan bench
re-run on a quiet Mac (01/02) and the list of 05/02.
