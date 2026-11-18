# Containers of an app's extensions in `app-leftovers`

**Agreed:** 2026-11-18 (plan 019 pre-flight) · **Due at:** plan 006, before `Library/Containers` is scanned

## What is postponed

Today `Library/Containers` is not opened (the places that prompt are avoided until plan 006), so no container is
offered or counts as proof. When plan 006 opens it, containers become offerable at High and prove an id with no change
in the module. An installed app's extension keeps its own container under its own id, which may be unrelated to the
app's (`InstalledApp::helpers` reads only one level of `LoginItems`, `XPCServices` and `PlugIns`); such a container
would then be offered while the app is installed.

## What doing it needs

Before containers are scanned: the helper list covers every nested bundle of an installed app, or the `container` row
of `modules/app-leftovers/places.toml` stays not offered. Check it by hand on a Mac with extensions of installed apps.
