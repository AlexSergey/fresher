# The disk image's window

**Agreed:** 2026-11-26, by the user, after opening the 0.1.0 dry-run image · **Due at:** 0.2.0

## What is postponed

The window the disk image opens with: today it shows `Fresher.app` and the `Applications` link on a plain background,
at the system's default size and places. The user wants it as polished as other apps' images: a background drawn for
it that says to drag Fresher to Applications (an arrow between the two icons), a fixed window size, and the two icons
placed on it.

## Why

0.1.0 was being tagged when it came up; the image installs as it is, and its look changes nothing of the app.

## What doing it needs

1. A background image in the app's style (light and dark legible; a Retina version), kept with the bundle's other
   resources under `app/tauri/`.
2. The bundle's disk-image settings in `app/tauri/tauri.conf.json` (`bundle.macOS.dmg`: background, window size, the
   app's and the Applications link's positions), checked against the Tauri version in use.
3. `npm run build:release` and `npm run check:bundle -- --release` pass; the image opened from a browser download
   shows the window as drawn, on macOS 13 and the newest.
4. The README's and the release notes' install steps still match what the window shows.
