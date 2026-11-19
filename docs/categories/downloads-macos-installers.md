# Old macOS installers

**Danger level:** Low · **Rule:** none (the module's code, `modules/downloads`) · **Since:** 0.1.0

## What it is

Installers of a macOS version this Mac already runs or has passed. Each takes 12 to 15 GB. Made by you or the App
Store, when you downloaded macOS.

## Where it lives

`~/Downloads`, first level only: a folder named in `modules/downloads/macos.toml` (`Install macOS Sonoma.app`) that
holds `Contents`, summed whole.

## What happens if you remove it

It goes to the Trash. To make a bootable installer later, download it again. It comes back only if you download it
again.

## How the app decides

The module `downloads`. A restored scan or a folder not read in full is left out (scan again); a macOS version that
cannot be read, left out; the running macOS older than the installer's version, left out (it is for a newer macOS);
a socket, pipe or device inside, or something not yours or locked, left out; otherwise **offered**. A name not in the
table is not claimed. `Install macOS ….app` in `/Applications` is never looked at. Nothing in Downloads is read unless
you turn on its switch in Settings or grant Full Disk Access. The folders go to the Trash.
