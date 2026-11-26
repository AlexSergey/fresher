# Cleanups in a sandbox, with the system's Trash

A development build in sandbox mode cleans only inside `sandbox/trial/` and moves items into the **system's** Trash
(`--real-trash`). Nothing outside the sandbox is touched; the Trash holds only the sandbox's items afterwards. How
sandbox mode works: [`../testing.md`](../testing.md#sandbox-mode).

## Setup

From the repository's root. The generated sandbox has the `trial` module's six cases
(`fixtures/sandboxes/trial.toml`, listed in `sandbox/trial/EXPECTED.md`); the rest is added by hand.

```sh
rm -rf sandbox/trial && npm run sandbox -- trial
cd sandbox/trial/home
M="$(head -1 ../.fresher-sandbox)"                       # the marker's signature line
for d in held finder-put-back permanent; do
  mkdir -p "$d/Sample app cache" && printf '%s\n' "$M" > "$d/.fresher-sandbox"
done
head -c 8388608   /dev/urandom > "held/Sample app cache/held.bin"            # 8 MB, held open below
head -c 41943040  /dev/urandom > "finder-put-back/Sample app cache/blob.bin" # 40 MB
head -c 104857600 /dev/urandom > "permanent/Sample app cache/blob.bin"      # 100 MB
cd ../../..
```

In a second terminal, keep a file open for the whole run (SC-02), and stop it with Ctrl-C at the end:

```sh
tail -f "sandbox/trial/home/held/Sample app cache/held.bin" > /dev/null
```

Start the app: `npm run dev:sandbox -- trial --real-trash`. The run is recorded in `NO_COMMIT/dev-runs/` (its
`.events.jsonl` shows every action and outcome: read it when a case is unclear).

Every cache has the same title, "Sample app cache"; tell them apart by size: about 105 MB `permanent`, 52 MB `work`,
42 MB `finder-put-back`, 8 MB `held`; `Old sample downloads` shows 0 B.

## Cases

### SC-01 The scan offers exactly the trial items

Scan. Offered: the four caches and `Old sample downloads`. Never offered: `look-alike/Sample app cache2`,
`bare/Sample app cache` (no marker), `named/Sample app cache.txt` (a file), `plain/keep-me.txt`.

### SC-02 A folder in use is left out, naming the program

Pick every cache but the 105 MB one and Review, then remove. The confirmation leaves out the 8 MB cache as in use and
names the program holding it (`tail`). It is not in the plan; it is still on disk afterwards.

### SC-03 Move to the system's Trash, same names side by side

Confirm the move to the Trash (the first time, accept the agreement). "Moved to the Trash" with the count and the
size; `work/` and `finder-put-back/` keep only their markers; Finder's Trash holds two "Sample app cache" items, the
second renamed by macOS.

### SC-04 Put back all from Fresher, each to its own place

Cleanups: the cleanup says all its items are in the Trash. Put back all. `work/Sample app cache` is back with its
`nested/` folder (50 MB), `finder-put-back/Sample app cache` with its 40 MB file: neither in the other's place. The
Trash has no "Sample app cache" left.

### SC-05 Delete permanently

Rescan, pick the 105 MB cache, delete permanently. "Freed" with about 105 MB; `permanent/` keeps only its marker;
nothing new in the Trash.

### SC-06 Empty a folder and keep it

Rescan, pick `Old sample downloads` and the 42 MB cache, move to the Trash. `keep/Old sample downloads/` stays and is
empty; `entry.bin` is in the Trash; the cache is in the Trash.

### SC-07 Review after a finished cleanup shows the new pick

Right after SC-06, without pressing Done on the result: rescan, pick a cache, open Review. Review shows the pick, not
the last cleanup's result. Picking or unpicking anything also clears the result.

### SC-08 Finder's own Put Back

In Finder's Trash, Put Back the 42 MB "Sample app cache" and `entry.bin`. Today Finder may put the folder in the wrong
place or refuse (another same-named folder was trashed and put back before), and offers no Put Back for an emptied
folder's file: [known issue](../known-issues/cleaner-finder-put-back-unreliable.md). Nothing is ever overwritten. Put
anything Finder misplaced back where it belongs by hand.

### SC-09 Put back one item, and an emptied folder's file, from Fresher

Cleanups: put back a single item of a cleanup (not all), and the file of the emptied folder (`entry.bin`). Each goes
back to its own place; the cleanup then shows how many are still in the Trash. An item put back by Finder (SC-08) is
shown as no longer in the Trash.

### SC-10 Quit during a long cleanup

Add 2 000 small caches, then rescan:

```sh
cd sandbox/trial/home && M="$(head -1 ../.fresher-sandbox)" && mkdir many
for i in $(seq -w 1 2000); do
  mkdir -p "many/$i/Sample app cache" && printf '%s\n' "$M" > "many/$i/.fresher-sandbox"
  head -c 2048 /dev/zero > "many/$i/Sample app cache/a.bin"
done
cd ../../..
```

A move to the Trash is one rename per item and ends in a second or two, too fast to quit during it; a permanent delete
removes every file. So make 300 caches of 1 000 files each instead of the 2 000 above:

```sh
cd sandbox/trial/home && python3 -I -c '
import os
sig = open("../.fresher-sandbox").readline()
for i in range(300):
    cache = os.path.join("slow", f"{i:03}", "Sample app cache")
    os.makedirs(cache)
    open(os.path.join("slow", f"{i:03}", ".fresher-sandbox"), "w").write(sig)
    for k in range(1000):
        open(os.path.join(cache, f"f{k:04}.bin"), "wb").write(b"x")
'
cd ../../..
```

Pick the whole cache category, choose **Delete permanently** in the confirmation and confirm; press ⌘Q right away. While
it runs: ⌘Q asks; **Keep cleaning** changes nothing and the cleanup goes on. ⌘Q again, **Stop and quit**: the app stops
after the item in hand and quits. Closing the window asks the same. The Dock's Quit does not ask: [known
issue](../known-issues/app-quit-during-cleanup.md).

### SC-11 After a stopped cleanup, Cleanups shows it stopped

Start the app again. Cleanups shows the cleanup of SC-10 as stopped, with how many items went and how many did not;
the items not reached are still on disk.

### SC-12 The disk chart after a cleanup

After SC-05 and SC-06, the chart on Scan shows the space the cleanups took out. Today it keeps the old numbers:
[known issue](../known-issues/ui-chart-stale-after-a-cleanup.md).

## Afterwards

Quit the app, stop the `tail` (Ctrl-C), empty the sandbox's items from the Trash by hand, and
`rm -rf sandbox/trial`.
