# The rescan property test fails at times on a file's access time

**Status:** fixed ([plan 017, Conclusions](../plans/017-temp-files/readme.md#conclusions)) · **Updated:** 2026-11-18

## Symptom

`rescan::patch::tests::property::a_rescan_equals_a_full_scan` (`crates/scanner/src/rescan/patch_tests.rs`) fails about
once in 1,500 cases, only under load (the whole workspace's tests running at once): a whole folder's
`inside.newest_file_access` differs by one second between the rescan and the full scan that follows it. Every other
value of the two trees is equal, `inside.lock_named` included. The failing seed does not fail again when run alone, so
no regression file is kept. Seen during plan 017/02/02 (2026-11-17); the field comes from plan
[017/01/03](../plans/017-temp-files/01-core-prerequisites/03-times.md).

## Cause

Not proven. The likely one: a file's access time moves while the two walks read the tree. macOS moves an access time
on a read only while it is older than the modification time; a file the test's random changes write in more than one
call, across a second, keeps its creation second as its access time, and the first walk that reads its content (a
marker such as `CACHEDIR.TAG`) moves it, so the rescan and the full scan see different seconds. The module that reads
the value (`temp`) only ever keeps an item on it, and a real scan reads it once, so a scan is never less careful
because of it.

## What was tried

- `fresher_testing::settle_access_time`: the fixture builders (`FixtureTree`, `LargeTree`, the scanner's integration
  homes) set every file's access time to its modification time, so a walk does not move it. This made the thread-count
  and the rescan comparisons stable; the property test's own random changes between the scans are not settled.
- A folder's own access time is never reported (`Match.accessed` is `None` for a folder) for the same reason.

## The fix

The property test's own changes (`change` in `patch_tests.rs`) settle the access time of every file they write, a new
file, a lengthened one and the file of a new folder, with `fresher_testing::settle_access_time`, as the fixtures do. No
value is left out of the comparison. After the fix the test passed 20 runs in a row
(`cargo nextest run -p fresher-scanner a_rescan_equals_a_full_scan --retries 0`, 64 cases each).

## Options left

None needed. Leaving `newest_file_access` out of the equality was the other option, kept in reserve should it fail
again.
