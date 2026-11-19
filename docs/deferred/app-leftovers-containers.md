# Containers of an app's extensions in `app-leftovers`

**Agreed:** 2026-11-18 (plan 019 pre-flight) · **Due at:** when `InstalledApp::helpers` covers every nested bundle

**Settled for plan 006 (2026-11-19):** the design's open question 1 took option (b): `modules/app-leftovers/places.toml`
has no `container` row, so the module does not read `Library/Containers` even when the walk may enter it (data from
other apps, or Full Disk Access). A container neither proves an id nor is offered, as before plan 006.

## What is postponed

Reading containers: as proof of an app's id, for the id's age, and as findings of the data category at High. It waits
because an installed app's extension keeps its own container under its own id, which may be unrelated to the app's
(`InstalledApp::helpers` reads only one level of `LoginItems`, `XPCServices` and `PlugIns`); such a container would
prove, and get offered, the files of an app that is still installed.

## What doing it needs

The helper list covers every nested bundle of an installed app; then the `container` row comes back to `places.toml`
(folder `containers`, `proof = true`, `optional = true`), its title `app-leftovers.title.container` comes back to
`modules/app-leftovers/locales/en.json`, the data category's texts and catalogue files name containers again, and its
three categories declare `Need::sees_more(Capability::OtherAppsData)` (an unseen container then hides a proof). Check it
by hand on a Mac with extensions of installed apps.
