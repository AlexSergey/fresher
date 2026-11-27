# Test cases by hand

The checks a person runs on a real Mac **before every release**: what the autotests cannot hold (the system's Trash,
Finder, the real process table, the real window, quitting). They are kept and grown from release to release; a run
by hand of a plan ([`../manual-testing.md`](../manual-testing.md)) is a one-off walk of new work, these are run again
every time. [`../release.md`](../release.md), "Cutting a release", step 5, runs them.

A bug found by a run gets its fix and an autotest where one can hold it, and a case here when only a person can see it.
A defect we keep for now goes to [`../known-issues/`](../known-issues/README.md) and its case says so.

## Files

One file per area, each with its setup, then its cases: an id, the steps, what must happen. The results of each run
are in the table below, one column per release.

| File                                     | Area                                                                               |
| ---------------------------------------- | ---------------------------------------------------------------------------------- |
| [`sandbox-cleanup.md`](./sandbox-cleanup.md) | scan, move to the Trash, put back, permanent delete, an item in use, quitting, in a sandbox with the system's Trash |
| [`high-and-largest.md`](./high-and-largest.md) | the "Only if you are sure" group, the High tick in the confirmation, Largest's ticks, an uninstall of everything, in a sandbox |
| [`left-out-lines.md`](./left-out-lines.md) | the left-out lines of a category's details, folded at the bottom, and a category with everything left out, on a real scan |

## Results

✔️ passed · ❌ failed (the link says what was done) · ⚠️ passed with a known issue · — not run

| Case  | What                                                     | 0.1.0                                                       | 0.2.0 |
| ----- | -------------------------------------------------------- | ----------------------------------------------------------- | ----- |
| SC-01 | The scan offers exactly the trial items                  | ✔️                                                          |       |
| SC-02 | A folder in use is left out, naming the program          | ✔️                                                          |       |
| SC-03 | Move to the system's Trash, same names side by side      | ✔️                                                          |       |
| SC-04 | Put back all from Fresher, each to its own place         | ✔️                                                          |       |
| SC-05 | Delete permanently                                       | ✔️                                                          |       |
| SC-06 | Empty a folder and keep it                               | ✔️                                                          |       |
| SC-07 | Review after a finished cleanup shows the new pick       | ❌ fixed by 6dbf5674, then ✔️                               |       |
| SC-08 | Finder's own Put Back                                    | ⚠️ [known issue](../known-issues/cleaner-finder-put-back-unreliable.md) |       |
| SC-09 | Put back one item, and an emptied folder's file, from Fresher | ✔️                                                     |       |
| SC-10 | Quit during a long cleanup: Keep cleaning, Stop and quit | ✔️; ⚠️ [known issue](../known-issues/fixed/ui-confirm-dialog-hides-the-method.md) |       |
| SC-11 | After a stopped cleanup, Cleanups shows it stopped       | ✔️                                                          |       |
| SC-12 | The disk chart after a cleanup                           | ⚠️ [known issue](../known-issues/fixed/ui-chart-stale-after-a-cleanup.md) |       |
| LO-01 | Many left-out items are folded at the bottom             | —                                                           |       |
| LO-02 | A category whose items are all left out shows no zeros   | —                                                           |       |
| HL-01 | The "Only if you are sure" group                         | —                                                           |       |
| HL-02 | The High tick in the confirmation                        | —                                                           |       |
| HL-03 | Largest's ticks                                          | —                                                           |       |
| HL-04 | Uninstall everything                                     | —                                                           |       |
