# Largest's folders on a Mac

What only a person can see of the Largest screen's **Folders** and **Browse**: the Finder opening on a folder, the
setting that decides which folders are listed, and a real home's list. The autotests hold the screens with a mocked
core; these cases run them on a real scan. Fresher never removes a folder listed here: nothing is ticked and nothing
is cleaned in any case below.

## Setup

From the repository's root, the development build in sandbox mode, for LF-01 to LF-04:

```sh
rm -rf sandbox/largest-folders && npm run dev:sandbox -- largest-folders    # LF-01, LF-02, LF-03
rm -rf sandbox/large-files     && npm run dev:sandbox -- large-files        # LF-04
```

The sandbox writes about 2.6 GB of filled data. Its `EXPECTED.md` lists every folder it plants, listed or not. Quit
the window before starting the next one. LF-05 runs on the real home with `npm run dev` (nothing is removed there
either).

## Cases

### LF-01 The folders left alone

`largest-folders`. Scan, open **Largest**, then the **Folders** tab. The heading says "Open these in the Finder and
decide what to keep", and the line under it "Folders of 500 MB or more where nothing changed for 365 days. Fresher
never removes them." The bar reads "{size} of your home's {size}", then "In this list" and three tiles: Folders 3,
Space about 1.6 GB, Projects 1. The list "Folders left alone, largest first" holds three rows, each about 520 MB and `Archive` and
`old-app` the same size, so those two come in path order: `Archive` (`~/Archive`, it holds `2019`), `old-app`
(`~/Code/old-app`, with the **Project** label), `old-export` (`~/Public/old-export`). Each says "Not changed for 13 months.
Open it in the Finder to decide." and has a **Show in Finder** button. Not listed: `~/Code`, `~/Archive/2019`,
`~/Public`, `~/Small`, the ignored `~/Old-tools` and the copied `~/Restored`. No row has a checkbox and the selection
bar does not show.

### LF-02 Show in Finder

`largest-folders`, on **Folders**. Press **Show in Finder** on `old-app` (its name for a screen reader is "Show in
Finder: old-app"): a Finder window opens on `Code` in the sandbox's home with `old-app` selected. Nothing is ticked, no
selection bar shows, and the folder is still on disk.

### LF-03 The settings

`largest-folders`, after the scan. **Settings**, group **Scan**, "Smallest folder on Largest": it says 500 MB (the
sandbox sets it; the default is 1 GB), with the options 500 MB, 1 GB, 5 GB and 10 GB. Set it to **1 GB**: Largest, Folders says "No folder of 1 GB or more has gone unchanged for 365 days." and shows no
tiles. Set it back to **500 MB**: the three rows come back as in LF-01, without a new scan.

### LF-04 Browse is read-only

`large-files`. Scan, **Largest**, the **Browse** tab. The list "Folders and files, largest first" shows the home's folders
and files under "Go into a folder, or show it in the Finder", the path line "Where you are" starting at **Home**. No row
has a checkbox. Press **Open** on a folder: its content is listed and the path line gains the folder; **Home** in the
path line comes back. Press **Show in Finder** on a file and on a folder: Finder opens with each selected. `Public`
says "A standard folder of your Mac. Open it to choose what is inside." and still has **Open**; `Library` has no
**Open**. Switching to **Files** and back to **Browse** keeps the folder you were in.

### LF-05 A real home

`npm run dev` (the real home, never a sandbox), scan, Largest, **Folders**. The list holds no standard folder of the
home (`Desktop`, `Documents`, `Downloads`, `Movies`, `Music`, `Pictures`, `Public`) itself, nothing in `~/Library`, no
app and no hidden folder. Pick one row: its age ("Not changed for N months" or years) agrees with the Finder's "Date
Modified" of the newest file inside that folder (look for it in the Finder by hand, sorting the folder's content by
that column, including inside its subfolders). **Show in Finder** on it opens the right folder. Nothing was removed.
