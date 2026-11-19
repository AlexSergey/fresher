# 026/03/01 — The checker

**Status:** ✔️ done · **Version:** 0.1.0 · **Date:** 2026-11-19 · **Modules:** scripts

`npm run dev-runs:check -- <record.json>` reads a record, checks every finding and every cleanup item with a path
against the disk without changing it, and writes `<record>.check.md` beside the record, never over an older report.
The agent runs it on the user's record. The design: [`../00-design.md`](../00-design.md), section 4 (agent's calls 12,
13 and 18).

## Files

- `scripts/dev-runs-check.ts` (new) — in the shape of `scripts/sandbox-dev.ts` (pure functions exported, the main
  block under `import.meta.main`, a `UsageError`); TypeScript strict, `type` only, `readonly` fields, no `any` (the
  record is `unknown` until narrowed):
  - `export type Completeness = 'complete' | 'partial' | 'unknown';`
  - `export type Item = { readonly source: 'finding' | 'clear'; readonly module: string; readonly category: string;
    readonly path: string; readonly allocated: number; readonly completeness: Completeness; readonly outcome: string |
    null; readonly scanAt: number }` (`scanAt`: the item's scan's `finished`, seconds since the epoch; a cleanup item
    takes its `scan`'s);
  - `export type RunRecord = { readonly file: string; readonly build: string; readonly mode: string; readonly home:
    string; readonly roots: readonly string[]; readonly settings: Readonly<Record<string, unknown>>; readonly items:
    readonly Item[] }`;
  - `export const parseRecord = (file: string, text: string): RunRecord`: `UsageError` for invalid JSON, a `format`
    other than `1`, or a missing `session`, `scans` or `clear`; a finding is taken when its `subject` is
    `{ kind: 'path', value }`; a `clear` item when its `path` is a string; an unfinished scan's items are skipped;
  - `export type Stat = { readonly kind: 'file' | 'folder' | 'link' | 'other'; readonly blocks: number; readonly dev:
    number; readonly ino: number; readonly mtimeMs: number; readonly ctimeMs: number };`
    `export type Disk = { readonly lstat: (path: string) => Stat | undefined; readonly list: (folder: string) =>
    readonly string[] };` and `export const nodeDisk: Disk` (`lstatSync` with `throwIfNoEntry: false`, `bigint`
    false; `readdirSync`; an unreadable folder lists nothing). Nothing else of `node:fs` is used but the report's
    write;
  - `export type Check = 'gone' | 'link' | 'kind' | 'size' | 'outside' | 'protected' | 'stale-projects' |
    'downloads' | 'temp' | 'changed-after-scan';`
    `export type Difference = { readonly path: string; readonly module: string; readonly category: string; readonly
    check: Check; readonly expected: string; readonly found: string };`
  - `export const checkRecord = (record: RunRecord, disk: Disk): readonly Difference[]`: the design's section 4 table,
    in the record's order, one difference per failed check of an item; `~` in a root read as `home`; the size walk
    counts `blocks × 512` of every entry below the item (links not followed, a `dev:ino` counted once), within 5 % or
    1 MiB (`1_048_576`) of `allocated`, only for `complete` sizes; a time after `scanAt` is `changed-after-scan`;
    `stale-projects.unused-for-days` and `temp.older-than-days` from `settings` (a missing or non-numeric value: the
    module's default, 365 and 7);
  - `export const renderReport = (record: RunRecord, differences: readonly Difference[]): string`: `# Check of
    <file>`, the build and the mode, the counts (findings checked, cleanup items checked, differences), then a Markdown
    table `| Path | Module | Category | Check | Expected | Found |` or `No differences.`;
  - `export const reportPath = (record: string, exists: (path: string) => boolean): string`: `<record>.check.md`, else
    `<record>.check-2.md`, `-3`, …;
  - the main block: one argument (else usage, exit 2), read, parse, check with `nodeDisk`, write the report with
    `writeFileSync(path, text, { flag: 'wx' })`, print its path, exit 0 without a difference and 1 with one.
- `scripts/dev-runs-check.test.ts` (new) — `node:test`; every tree is made under `mkdtempSync(join(tmpdir(),
  'dev-runs-check-'))` with `writeFileSync`, `mkdirSync`, `symlinkSync`, `linkSync` and `utimesSync`; the records are
  built in the test as objects in the record's shape; nothing outside the temporary folder is read but `/` for the
  protected-place rows (`lstat` only).
- `package.json` — `"dev-runs:check": "node scripts/dev-runs-check.ts"`.

## Produces

`npm run dev-runs:check`, for 03/02's checklist and 03/03's run.

## Checks

- `scripts/dev-runs-check.test.ts`:
  - `describe('negative cases')`:
    `a_record_of_another_format_is_refused`, `a_file_that_is_not_json_is_refused`,
    `a_missing_finding_is_a_difference` (`gone`), `a_link_in_place_of_a_folder_is_a_difference` (`link`),
    `a_socket_or_pipe_is_a_difference` (`kind`; a named pipe made with `mkfifo` through `spawnSync`, skipped when it
    cannot be made), `a_size_beyond_the_tolerance_is_a_difference` (`size`; within it is not),
    `a_partial_size_is_not_compared`, `a_path_outside_the_roots_is_a_difference` (`outside`; a look-alike prefix
    `<root>2/x` is outside), `the_home_or_its_standard_folder_itself_is_a_difference` (`protected`),
    `a_stale_project_with_a_recent_folder_or_branch_is_a_difference` (one with a new `src/` folder, one with a new
    `.git/refs/heads`), `an_artifact_named_src_or_under_library_or_an_app_is_a_difference`,
    `a_rust_cache_built_within_seven_days_is_a_difference`, `a_download_below_the_first_level_is_a_difference`,
    `a_temp_item_changed_within_the_days_is_a_difference`, `a_moved_item_still_in_place_is_a_difference`,
    `an_existing_report_is_never_overwritten` (`reportPath` answers `-2` when `.check.md` exists, `-3` when both do);
  - `describe('positive cases')`:
    `a_record_that_matches_the_disk_has_no_difference` (a stale project's `node_modules`, a download, a temp item, a
    `would-move` item, all dated before the days), `hard_links_are_counted_once`,
    `a_change_after_the_scan_is_reported_as_such` (`changed-after-scan`, not `stale-projects`),
    `an_item_without_a_path_is_not_checked` (a tool's cleanup item, an external finding),
    `the_report_lists_every_difference_in_a_table`, `the_report_says_when_there_is_no_difference`,
    `the_settings_days_are_used_and_the_defaults_without_them`.
- `npm run test:scripts`; `npm run lint:ts`; `npx eslint scripts/dev-runs-check.ts scripts/dev-runs-check.test.ts`;
  `npm run knip`; `npx prettier --check` on the changed files.
- By hand once, after plan 02 is in: `npm run dev-runs:check -- <a record of a sandbox run>` (for example after `npm
  run dev:sandbox -- trial`, a scan and a cleanup): the moved items are not differences (they are gone, as `moved`
  says); a difference the sandbox's layout explains (its temporary places are not the Mac's) is written down as such
  in the conclusions.

## Conclusions

Done. `scripts/dev-runs-check.ts` and its test (24 cases) as the brief says, with these rulings of the preflight and
these choices where the brief was silent:

- A finding listed in its scan's `removed` is not checked as a finding (its `moved` cleanup item carries the check);
  `settings` are read as plain values; the pipe test uses a fake `Disk` answering kind `other`.
- An item ended `moved` or `deleted` must be gone (`gone` when it is still there); `would-*`, `skipped`, `failed` and
  `not-reached` items must still exist; `emptied`, `tool-done` and `unknown` have no presence check.
- A cleanup item whose scan is missing takes the last finished scan's time; a scan that has not finished gives nothing.
- The forbidden components `.git`, `src` and `dist` are read from the path below the home (a project kept under
  `~/src` is not a difference by itself).
- A project's activity is the `mtime` of its folders other than the artifacts and `.git` (not the project folder
  itself), plus the `.git/refs/heads` folders; a cache's, of every folder in it; a temporary item's, the `mtime` and
  `ctime` of every entry. A time after the scan is `changed-after-scan`, else within the days is the module's check.
- Following a link or a non-regular entry reports it and skips the size walk and the module rules.
